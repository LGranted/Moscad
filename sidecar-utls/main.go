package main

import (
	"bufio"
	"bytes"
	"crypto/tls"
	"encoding/binary"
	"flag"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"strings"
	"sync"
	"syscall"
	"time"

	utls "github.com/refraction-networking/utls"
	"golang.org/x/net/http2"
)

const (
	defaultSockPath              = "/data/data/com.lbjlaq.antigravity_tools/files/utls.sock"
	chromeConnWindowIncrement    = uint32(15663105) // 15728640 − 65535
	h2PrefaceStr                 = "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
	h2PrefaceLen                 = 24
	h2FrameHeaderLen             = 9
)

// Chrome 133 SETTINGS: HEADER_TABLE_SIZE=65536, ENABLE_PUSH=0,
// INITIAL_WINDOW_SIZE=6291456, MAX_HEADER_LIST_SIZE=262144
type h2Setting struct{ id uint16; val uint32 }

var chromeSettings = []h2Setting{
	{0x1, 65536}, {0x2, 0}, {0x4, 6291456}, {0x6, 262144},
}

var sockPath string

func main() {
	flag.StringVar(&sockPath, "socket", defaultSockPath, "Unix Domain Socket path")
	flag.Parse()

	os.Remove(sockPath)
	ln, err := net.Listen("unix", sockPath)
	if err != nil {
		log.Fatalf("[sidecar] Listen %s: %v", sockPath, err)
	}
	defer ln.Close()
	os.Chmod(sockPath, 0600)
	log.Printf("[sidecar] uTLS/Chrome_Auto H2 proxy ready on %s", sockPath)

	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGTERM, syscall.SIGINT)
	go func() {
		<-sigCh
		ln.Close()
		os.Remove(sockPath)
		os.Exit(0)
	}()

	for {
		conn, err := ln.Accept()
		if err != nil {
			if strings.Contains(err.Error(), "use of closed network connection") {
				return
			}
			log.Printf("[sidecar] Accept: %v", err)
			continue
		}
		go handleConn(conn)
	}
}

var (
	transportsMu sync.Mutex
	transports   = make(map[string]*http2.Transport)
)

func getTransport(hostport string) *http2.Transport {
	transportsMu.Lock()
	defer transportsMu.Unlock()
	if t, ok := transports[hostport]; ok {
		return t
	}
	hp := hostport
	t := &http2.Transport{
		// ИСПРАВЛЕНО: Правильная сигнатура DialTLS для свежего Go
		DialTLS: func(network, addr string, cfg *tls.Config) (net.Conn, error) {
			return dialChrome(hp)
		},
	}
	transports[hostport] = t
	return t
}

func dialChrome(addr string) (net.Conn, error) {
	host, port, err := net.SplitHostPort(addr)
	if err != nil {
		host, port = addr, "443"
	}
	rawConn, err := net.DialTimeout("tcp", net.JoinHostPort(host, port), 15*time.Second)
	if err != nil {
		return nil, fmt.Errorf("tcp %s: %w", addr, err)
	}
	if tc, ok := rawConn.(*net.TCPConn); ok {
		tc.SetKeepAlive(true)
		tc.SetKeepAlivePeriod(60 * time.Second)
	}

	// ИСПРАВЛЕНО: HelloChrome_Auto сам подтянет свежий Chrome (133+) и ML-KEM
	tlsConn := utls.UClient(rawConn, &utls.Config{ServerName: host}, utls.HelloChrome_Auto)
	if err := tlsConn.Handshake(); err != nil {
		rawConn.Close()
		return nil, fmt.Errorf("utls handshake %s: %w", host, err)
	}
	if tlsConn.ConnectionState().NegotiatedProtocol != "h2" {
		tlsConn.Close()
		return nil, fmt.Errorf("%s: no h2 negotiated", host)
	}
	return &chromeConn{Conn: tlsConn}, nil
}

type chromeConn struct {
	net.Conn
	mu       sync.Mutex
	buf      bytes.Buffer
	injected bool
}

func (c *chromeConn) Write(b []byte) (int, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.injected {
		return c.Conn.Write(b)
	}
	c.buf.Write(b)
	data := c.buf.Bytes()
	if len(data) < h2PrefaceLen+h2FrameHeaderLen {
		return len(b), nil
	}
	if string(data[:h2PrefaceLen]) != h2PrefaceStr {
		c.injected = true
		out := make([]byte, len(data))
		copy(out, data)
		c.buf.Reset()
		_, err := c.Conn.Write(out)
		return len(b), err
	}
	sh := data[h2PrefaceLen : h2PrefaceLen+h2FrameHeaderLen]
	payloadLen := int(sh[0])<<16 | int(sh[1])<<8 | int(sh[2])
	totalOrig := h2PrefaceLen + h2FrameHeaderLen + payloadLen
	if len(data) < totalOrig {
		return len(b), nil
	}
	tail := make([]byte, len(data)-totalOrig)
	copy(tail, data[totalOrig:])
	c.buf.Reset()
	c.injected = true

	var out bytes.Buffer
	out.WriteString(h2PrefaceStr)
	out.Write(buildSettingsFrame(chromeSettings))
	out.Write(buildWindowUpdateFrame(0, chromeConnWindowIncrement))
	out.Write(tail)
	_, err := c.Conn.Write(out.Bytes())
	return len(b), err
}

func buildSettingsFrame(ss []h2Setting) []byte {
	payload := make([]byte, 6*len(ss))
	for i, s := range ss {
		binary.BigEndian.PutUint16(payload[i*6:], s.id)
		binary.BigEndian.PutUint32(payload[i*6+2:], s.val)
	}
	pLen := len(payload)
	frame := make([]byte, h2FrameHeaderLen+pLen)
	frame[0], frame[1], frame[2] = byte(pLen>>16), byte(pLen>>8), byte(pLen)
	frame[3] = 0x4
	copy(frame[h2FrameHeaderLen:], payload)
	return frame
}

func buildWindowUpdateFrame(streamID, inc uint32) []byte {
	f := make([]byte, h2FrameHeaderLen+4)
	f[0], f[1], f[2] = 0, 0, 4; f[3] = 0x8
	binary.BigEndian.PutUint32(f[5:], streamID&0x7FFFFFFF)
	binary.BigEndian.PutUint32(f[9:], inc&0x7FFFFFFF)
	return f
}

func handleConn(clientConn net.Conn) {
	defer clientConn.Close()
	clientConn.SetDeadline(time.Now().Add(120 * time.Second))

	req, err := http.ReadRequest(bufio.NewReaderSize(clientConn, 64*1024))
	if err != nil { return }
	defer req.Body.Close()

	targetHost := req.Host
	if targetHost == "" && req.URL != nil { targetHost = req.URL.Host }
	if targetHost == "" { writeError(clientConn, 400, "missing Host"); return }
	hostport := targetHost
	if !strings.Contains(hostport, ":") { hostport += ":443" }

	upURL := &url.URL{Scheme: "https", Host: targetHost, Path: req.URL.Path, RawQuery: req.URL.RawQuery}
	outReq, err := http.NewRequest(req.Method, upURL.String(), req.Body)
	if err != nil { writeError(clientConn, 500, err.Error()); return }
	for k, vv := range req.Header {
		switch strings.ToLower(k) {
		case "proxy-connection", "proxy-authorization", "te", "trailers", "transfer-encoding", "upgrade": continue
		}
		for _, v := range vv { outReq.Header.Add(k, v) }
	}
	outReq.Host = targetHost

	resp, err := getTransport(hostport).RoundTrip(outReq)
	if err != nil {
		log.Printf("[sidecar] RoundTrip %s: %v", upURL, err)
		writeError(clientConn, 502, err.Error()); return
	}
	defer resp.Body.Close()

	st := http.StatusText(resp.StatusCode)
	if st == "" { st = "Unknown" }
	fmt.Fprintf(clientConn, "HTTP/1.1 %d %s\r\n", resp.StatusCode, st)
	for k, vv := range resp.Header {
		for _, v := range vv { fmt.Fprintf(clientConn, "%s: %s\r\n", k, v) }
	}
	fmt.Fprintf(clientConn, "\r\n")
	io.Copy(clientConn, resp.Body)
}

func writeError(w io.Writer, code int, msg string) {
	body := fmt.Sprintf(`{"error":%q}`, msg)
	fmt.Fprintf(w, "HTTP/1.1 %d %s\r\nContent-Type: application/json\r\nContent-Length: %d\r\n\r\n%s",
		code, http.StatusText(code), len(body), body)
}
