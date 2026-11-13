// doc/testing/web_server.md#frontend-unit-tests-unit

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { checkValueRule } from '../js/views/settings_rules.js';

const RULES = {
  export_dir: 'folder',
  camera_fps: 'integer 1..=120',
  camera_source: 'one of v4l2, rtsp, none',
  camera_rtsp_url: 'URL starting with rtsp://',
  web_bind_address: '<IPv4 address>:<port>',
  camera_controls: 'JSON object',
  camera_device: 'text',
  live_feed_quality: 'integer 1..=100',
  admin_password_hash: 'Argon2id hash (PHC string)',
};

// WS-JS-08
test('WS-JS-08', () => {
  const valid = [['export_dir', '/srv/exports'], ['camera_fps', '1'], ['camera_fps', '120'], ['camera_source', 'rtsp'],
    ['camera_rtsp_url', 'rtsp://192.168.1.5/stream'], ['web_bind_address', '192.168.1.10:80'], ['camera_controls', '{}'], ['camera_device', 'usb-1']];
  const invalid = [['camera_fps', '0'], ['camera_fps', '121'], ['camera_fps', 'abc'], ['camera_fps', '30.5'], ['camera_source', 'usb'],
    ['camera_rtsp_url', 'http://x'], ['web_bind_address', '8080'], ['web_bind_address', '0.0.0.0:70000'], ['web_bind_address', 'localhost:8080'],
    ['camera_controls', '[]'], ['camera_controls', '{'], ['live_feed_quality', '101'], ['admin_password_hash', 'secret']];
  for (const [name, value] of valid) assert.equal(checkValueRule(RULES[name], value), null, `${name}=${value}`);
  for (const [name, value] of invalid) assert.notEqual(checkValueRule(RULES[name], value), null, `${name}=${value}`);
});
