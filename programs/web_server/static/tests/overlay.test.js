// doc/testing/web_server.md#frontend-unit-tests-unit

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { displayedToImage } from '../js/overlay.js';

// WS-JS-03
test('WS-JS-03', () => {
  assert.deepEqual(displayedToImage({ x: 320, y: 180 }, 640, 1920), { x: 960, y: 540 });
});
