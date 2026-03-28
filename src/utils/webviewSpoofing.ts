// WebView spoofing — подмена браузерных fingerprint для защиты от Google детекторов
export function injectWebViewSpoofing() {
  if (typeof window === 'undefined') return;

  // 1. Navigator spoofing
  const chromeVersion = '131';
  Object.defineProperty(navigator, 'userAgent', {
    get: () => `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/${chromeVersion}.0.0.0 Mobile Safari/537.36`,
    configurable: true,
  });

  Object.defineProperty(navigator, 'appVersion', {
    get: () => `5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/${chromeVersion}.0.0.0 Mobile Safari/537.36`,
    configurable: true,
  });

  Object.defineProperty(navigator, 'platform', {
    get: () => 'Linux aarch64',
    configurable: true,
  });

  Object.defineProperty(navigator, 'hardwareConcurrency', {
    get: () => 8,
    configurable: true,
  });

  Object.defineProperty(navigator, 'deviceMemory', {
    get: () => 8,
    configurable: true,
  });

  Object.defineProperty(navigator, 'languages', {
    get: () => ['en-US', 'en'],
    configurable: true,
  });

  // 2. Canvas fingerprint noise
  const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
  HTMLCanvasElement.prototype.toDataURL = function(...args) {
    const ctx = this.getContext('2d');
    if (ctx) {
      const imageData = ctx.getImageData(0, 0, this.width, this.height);
      for (let i = 0; i < imageData.data.length; i += 4) {
        imageData.data[i] ^= 1;
      }
      ctx.putImageData(imageData, 0, 0);
    }
    return originalToDataURL.apply(this, args);
  };

  // 3. WebGL vendor/renderer spoofing
  const originalGetParameter = WebGLRenderingContext.prototype.getParameter;
  WebGLRenderingContext.prototype.getParameter = function(parameter) {
    if (parameter === 37445) return 'Google Inc. (Qualcomm)';
    if (parameter === 37446) return 'ANGLE (Qualcomm, Adreno (TM) 650, OpenGL ES 3.2)';
    return originalGetParameter.call(this, parameter);
  };

  // 4. Screen properties
  Object.defineProperty(screen, 'colorDepth', { get: () => 24, configurable: true });
  Object.defineProperty(screen, 'pixelDepth', { get: () => 24, configurable: true });

  // 5. Chrome object (важно для детекторов)
  if (!(window as any).chrome) {
    (window as any).chrome = {
      runtime: {},
      loadTimes: () => {},
      csi: () => {},
      app: {},
    };
  }

  // 6. Plugins (Chrome имеет пустой массив на мобильном)
  Object.defineProperty(navigator, 'plugins', {
    get: () => [],
    configurable: true,
  });

  // 7. WebRTC — скрываем реальный IP
  const originalRTCPeerConnection = (window as any).RTCPeerConnection;
  if (originalRTCPeerConnection) {
    (window as any).RTCPeerConnection = function(...args: any[]) {
      const pc = new originalRTCPeerConnection(...args);
      return pc;
    };
  }
}
