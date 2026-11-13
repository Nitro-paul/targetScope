// doc/testing/web_server.md#frontend-unit-tests-unit

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { generateRings, validateTargetType, paperToShot, shotLabel, shotLabels } from '../js/target.js';

const dir = new URL('../../../../testdata/target_types/', import.meta.url).pathname;
const ttRef = JSON.parse(readFileSync(join(dir, 'tt_ref.json'), 'utf8')).target_type;

// WS-JS-01
test('WS-JS-01', () => {
  const files = readdirSync(dir).filter((f) => f.endsWith('.json'));
  assert.ok(files.length > 0);
  for (const f of files) {
    const v = JSON.parse(readFileSync(join(dir, f), 'utf8'));
    assert.equal(validateTargetType(v.target_type) === null, v.valid, f);
    if (v.valid) {
      const rings = generateRings(v.target_type);
      assert.equal(rings.length, v.expected_rings.length, f);
      rings.forEach((r, i) => {
        assert.equal(r.score, v.expected_rings[i].score);
        assert.ok(Math.abs(r.radius_mm - v.expected_rings[i].radius_mm) < 1e-9);
      });
    }
  }
});

// WS-JS-02
test('WS-JS-02', () => {
  const s = paperToShot(ttRef, 110, 100);
  assert.ok(Math.abs(s.angle_turns - 0.25) < 1e-12);
  assert.ok(Math.abs(s.radius_mm - 10) < 1e-12);
});

// WS-JS-05
test('WS-JS-05', () => {
  assert.equal(shotLabel(1, 1, 3), '3');
  assert.equal(shotLabel(2, 2, 3), '2.3');
  const shots = [{ paper_no: 1 }, { paper_no: 2 }, { paper_no: 2 }, { paper_no: 2 }];
  assert.deepEqual(shotLabels(shots, 2), ['1.1', '2.1', '2.2', '2.3']);
});
