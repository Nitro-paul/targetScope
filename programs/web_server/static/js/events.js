// WebSocket /ws with reconnect (growing delays) and event dispatch by type.
// After a reconnect the views reload their state.

const listeners = new Map(); // type -> Set<fn(payload, envelope)>

export function on(type, fn) {
  if (!listeners.has(type)) listeners.set(type, new Set());
  listeners.get(type).add(fn);
  return () => listeners.get(type).delete(fn);
}

export function connectEvents() {
  // TODO
}
