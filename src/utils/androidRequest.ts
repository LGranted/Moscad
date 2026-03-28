import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const isAndroid = () => {
  return typeof window !== 'undefined' && 
    !!(window as any).__TAURI_INTERNALS__ && 
    /Android/i.test(navigator.userAgent);
};

export async function androidStealthRequest(
  url: string,
  options: RequestInit = {}
): Promise<Response> {
  const method = options.method || 'GET';
  const headers: Record<string, string> = {};
  
  if (options.headers) {
    Object.entries(options.headers as Record<string, string>).forEach(([k, v]) => {
      headers[k] = v;
    });
  }

  let body: number[] = [];
  if (options.body) {
    const encoder = new TextEncoder();
    body = Array.from(encoder.encode(options.body as string));
  }

  const result = await invoke<number[]>('handle_android_stealth_request', {
    url,
    method,
    headers,
    body,
  });

  const bytes = new Uint8Array(result);
  return new Response(bytes);
}

export async function androidStealthStream(
  url: string,
  options: RequestInit = {},
  onChunk: (chunk: string) => void,
  onDone: () => void,
  onError: (err: string) => void
): Promise<void> {
  const eventId = Math.random().toString(36).slice(2);
  const method = options.method || 'GET';
  const headers: Record<string, string> = {};

  if (options.headers) {
    Object.entries(options.headers as Record<string, string>).forEach(([k, v]) => {
      headers[k] = v;
    });
  }

  let body: number[] = [];
  if (options.body) {
    const encoder = new TextEncoder();
    body = Array.from(encoder.encode(options.body as string));
  }

  const unlistenChunk = await listen<string>(`stream-chunk-${eventId}`, (e) => onChunk(e.payload));
  const unlistenDone = await listen<string>(`stream-done-${eventId}`, () => {
    unlistenChunk();
    unlistenDone();
    unlistenError();
    onDone();
  });
  const unlistenError = await listen<string>(`stream-error-${eventId}`, (e) => {
    unlistenChunk();
    unlistenDone();
    unlistenError();
    onError(e.payload);
  });

  await invoke('handle_android_stealth_request_stream', {
    url,
    method,
    headers,
    body,
    eventId,
  });
}

export { isAndroid };
