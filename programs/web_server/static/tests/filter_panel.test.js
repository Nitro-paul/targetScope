// doc/testing/web_server.md#frontend-unit-tests-unit

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { filterToQuery, queryToFilter } from '../js/views/filter_panel.js';

// WS-JS-04
test('WS-JS-04', () => {
  const filters = [
    {},
    {
      session_ids: [1, 2],
      shooter_ids: [4],
      device_ids: [],
      device_categories: ['air pistol', 'x;y&z'],
      calibers: ['.22 LR'],
      target_type_ids: [3],
      distance_m: { min: 10, max: 25 },
      saved_at: { from: 1767225600 },
      single_handed: true,
      supported: false,
      shot_count: { max: 60 },
      include_retired: false,
    },
    { shooter_ids: [], distance_m: {} },
  ];
  for (const f of filters) assert.deepEqual(queryToFilter(filterToQuery(f)), f);
});
