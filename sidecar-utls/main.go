package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"os"
	"strings"
	"time"

	utls "github.com/refraction-networking/utls"
	"golang.org/x/net/http2"
)

func main() {
	sockPath := "/data/data/com.lbjlaq.antigravity_tools/files/utls.sock"
	os.Remove(sockPath)

	ln, err := net.Listen("unix", sockPath)
	if err != nil {
		log.Fatal(err)
	}
	os.Chmod(sockPath, 0777)

	// Настройка HTTP/2 транспорта с отпечатком Chrome 133
	h2Transport := &http2.Transport{
		AllowHTTP: true,
		Settings: []http2.Setting{
			{ID: http2.SettingHeaderTableSize, Val: 65536},
			{ID: http2.SettingEnablePush, Val: 0},
			{ID: http2.SettingMaxConcurrentStreams, Val: 1000},
			{ID: http2.SettingInitialWindowSize, Val: 6291456},
			{ID: http2.SettingMaxFrameSize, Val: 16384},
			{ID: http2.SettingMaxHeaderListSize, Val: 262144},
		},
	}

	log.Printf("Engine Started: Mimicking Chrome 133 (TLS + H2)")

	for {
		conn, err := ln.Accept()
		if err != nil {
			continue
		}
		go handleProxy(conn, h2Transport)
	}
}

func handleProxy(client net.Conn, h2 *http2.Transport) {
	defer client.Close()
	reader := bufio.NewReader(client)
	req, err := http.ReadRequest(reader)
	if err != nil {
		return
	}

	// Формируем запрос к реальному серверу
	targetURL := fmt.Sprintf("https://%s%s", req.Host, req.URL.RequestURI())
	outReq, _ := http.NewRequest(req.Method, targetURL, req.Body)
	for k, v := range req.Header {
		outReq.Header[k] = v
	}

	// uTLS Handshake
	rawConn, _ := net.DialTimeout("tcp", req.Host+":443", 10*time.Second)
	uConn := utls.UClient(rawConn, &utls.Config{ServerName: req.Host}, utls.HelloChrome_Auto)
	uConn.Handshake()

	var resp *http.Response
	if uConn.ConnectionState().NegotiatedProtocol == "h2" {
		c2, _ := h2.NewClientConn(uConn)
		resp, err = c2.RoundTrip(outReq)
	} else {
		// Fallback to H1
		resp, err = http.DefaultClient.Do(outReq)
	}

	if err == nil {
		resp.Write(client)
	}
}
