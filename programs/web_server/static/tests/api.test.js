// doc/testing/web_server.md#frontend-unit-tests-unit

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { request } from '../js/api.js';

// WS-JS-06
test('WS-JS-06', async () => {
  const keys = [];
  let calls = 0;
  const fetchFn = async (_path, init) => {
    keys.push(init.headers['Idempotency-Key']);
    calls += 1;
    if (calls === 1) throw new TypeError('network error');
    return { ok: true, status: 200, text: async () => '{"ok":true}' };
  };
  const r = await request('POST', '/api/session/shots', { angle_turns: 0.25, radius_mm: 12 }, { fetchFn });
  assert.deepEqual(r, { ok: true });
  assert.equal(calls, 2);
  assert.ok(keys[0]);
  assert.equal(keys[0], keys[1]);
});
