// doc/testing/web_server.md#frontend-unit-tests-unit

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { connectEvents, onReload } from '../js/events.js';

// WS-JS-07
test('WS-JS-07', () => {
  const sockets = [];
  class FakeSocket {
    constructor() {
      sockets.push(this);
    }
    close() {
      this.onclose?.();
    }
  }
  const pending = [];
  let reloads = 0;
  onReload(() => (reloads += 1));
  const c = connectEvents({ url: 'ws://x/ws', WebSocketImpl: FakeSocket, setTimeoutFn: (fn) => pending.push(fn) });
  sockets[0].onopen();
  for (let i = 0; i < 3; i++) {
    sockets.at(-1).close();
    pending.shift()();
  }
  assert.equal(sockets.length, 4);
  const d = c.state.delays;
  assert.ok(d[0] < d[1] && d[1] < d[2], JSON.stringify(d));
  sockets.at(-1).onopen();
  assert.equal(reloads, 1);
});
