// doc/programs/web_server.md#websocket-events

const listeners = new Map();
const reloaders = new Set();

export function on(type, fn) {
  if (!listeners.has(type)) listeners.set(type, new Set());
  listeners.get(type).add(fn);
  return () => listeners.get(type)?.delete(fn);
}

export function onReload(fn) {
  reloaders.add(fn);
  return () => reloaders.delete(fn);
}

export function dispatch(message) {
  const event = typeof message === 'string' ? JSON.parse(message) : message;
  if (event.type === 'resync') {
    reloaders.forEach((fn) => fn());
    return;
  }
  listeners.get(event.type)?.forEach((fn) => fn(event.payload, event));
  listeners.get('*')?.forEach((fn) => fn(event.payload, event));
}

export const BASE_DELAY_MS = 500;
export const MAX_DELAY_MS = 10000;

// doc/programs/web_server.md#websocket-events
export function connectEvents({ url, WebSocketImpl = globalThis.WebSocket, setTimeoutFn = globalThis.setTimeout } = {}) {
  const target = url ?? `${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/ws`;
  const state = { delays: [], socket: null, attempts: 0, stopped: false };
  const open = () => {
    if (state.stopped) return;
    const ws = new WebSocketImpl(target);
    state.socket = ws;
    ws.onopen = () => {
      if (state.attempts > 0) reloaders.forEach((fn) => fn());
      state.attempts = 0;
    };
    ws.onmessage = (m) => dispatch(m.data);
    ws.onclose = () => {
      const delay = Math.min(MAX_DELAY_MS, BASE_DELAY_MS * 2 ** state.attempts);
      state.attempts += 1;
      state.delays.push(delay);
      setTimeoutFn(open, delay);
    };
  };
  open();
  return { state, stop: () => { state.stopped = true; state.socket?.close(); } };
}
