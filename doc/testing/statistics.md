# Tests: statistics

Tests of the program `statistics`.

Design: [statistics.md](../programs/statistics.md), [interfaces.md](../interfaces.md#statistics). Reference data and tools: [overview.md](overview.md).

## Setup
- **Unit tests** call the functions of `calc.rs` and `periods.rs` directly.
- **Integration tests** run `statistics` as a process with `RedisTestServer` and either a `FakeProgram` as `database_handler` (to check the requests it sends) or a real `database_handler` with a fixture database.
- Periods are calculated in local time; tests set the time zone to `Europe/Berlin`.

## Values per session and group (unit)
What is tested: the formulas of [StatisticsValues](../interfaces.md#statisticsvalues) and [statistics.md](../programs/statistics.md#calculation-flow-compute_statistics).

| ID | Case | Input | Expected outcome |
|---|---|---|---|
| ST-CAL-01 | Extreme spread | Shots at (0, 0), (6, 8), (0, 8) mm | 10 |
| ST-CAL-02 | Extreme spread of one shot | One shot | 0 |
| ST-CAL-03 | Mean radius | Shots at (−1, 0), (1, 0) | Mean point (0, 0); mean radius 1 |
| ST-CAL-04 | Mean radius, uneven | Shots at (0, 0), (6, 8), (0, 8) | Mean point (2, 5.333); mean radius 4.612 (±0.001) |
| ST-CAL-05 | Score values | Session A: score 50, 5 shots; session B: score 90, 10 shots | `score_avg` 70; `score_min` 50; `score_max` 90; `score_per_shot_avg` 140 / 15 = 9.333 |
| ST-CAL-06 | Grouping per shot | A: grouping 55, 5 shots; B: grouping 100, 10 shots | `grouping_score_per_shot_avg` 155 / 15 = 10.333 |
| ST-CAL-07 | Mean point average | Mean points A (1, 2), B (3, −2) | `mean_point_avg` (2, 0) |
| ST-CAL-08 | Spread and radius per session | A: extreme spread 10, mean radius 3; B: 20, 5 | `extreme_spread_mm_avg` 15; `mean_radius_mm_avg` 4 (not weighted by shots) |
| ST-CAL-09 | Counts | A and B | `session_count` 2; `shot_count` 15 |

## Groups (integration)
What is tested: splitting by target type and by `group_by` ([interfaces.md](../interfaces.md#statisticsgroup)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| ST-GRP-01 | No grouping | `group_by` `none`; sessions on one target type | One group; `group_key` null |
| ST-GRP-02 | Always split by target type | `group_by` `none`; sessions on `TT-REF` and `TT-2` | Two groups, sorted by target type name |
| ST-GRP-03 | By shooter | `group_by` `shooter`; sessions of Alex, Bea and without shooter | Groups `{ shooter_id: Alex }`, `{ shooter_id: Bea }`, `{ shooter_id: null }` |
| ST-GRP-04 | By device | `group_by` `device` | Groups `{ device_id }` |
| ST-GRP-05 | By device category | `group_by` `device_category` | `read_devices` with `id in` the used devices; groups `{ category }` |
| ST-GRP-06 | By distance | `group_by` `distance`, including sessions without distance | Groups `{ distance_m }`, one with `distance_m` null |
| ST-GRP-07 | By single-handed and supported | `group_by` `single_handed`; then `supported` | Groups with `true` and `false` |
| ST-GRP-08 | Shots of a group | A group with 6 000 shots | `shots` has 5 000 shots, all from the newest sessions |

## Periods (unit)
What is tested: the trend periods in local time ([statistics.md](../programs/statistics.md#calculation-flow-compute_statistics)). The cases call `periods::period_start` directly; the session times are local times in `Europe/Berlin`.

| ID | Case | Sessions (local time) | Expected outcome |
|---|---|---|---|
| ST-PER-01 | One point per session | `period` `session`, 3 sessions | 3 trend entries; `period_start` = `saved_at` |
| ST-PER-02 | Days | `period` `day`: 2026-03-02 23:30 and 2026-03-03 00:30 | 2 entries |
| ST-PER-03 | Weeks start on Monday | `period` `week`: 2026-03-02 (Mon) and 2026-03-08 (Sun); 2026-03-08 (Sun) and 2026-03-09 (Mon) | Same week; different weeks |
| ST-PER-04 | Week across the year | 2026-12-31 (Thu) and 2027-01-02 (Sat) | One week, starting 2026-12-28 |
| ST-PER-05 | Daylight saving time | `period` `day`: 2026-03-29 01:30 and 03:30 (clocks change at 02:00); 2026-03-30 00:30 | First two on one day, the third on the next |
| ST-PER-06 | Months | `period` `month`: 2026-01-31 and 2026-02-01 | 2 entries |
| ST-PER-07 | Empty periods | Sessions in January and March | No entry for February |

## Flow and requests (integration)
What is tested: what `statistics` asks `database_handler` and what it answers ([statistics.md](../programs/statistics.md#calculation-flow-compute_statistics)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| ST-FLW-01 | Filter passed on | `compute_statistics` with a SessionFilter using every field | `read_sessions` receives the same filter, `include_shots` true, `order_by` `id asc` |
| ST-FLW-02 | Paging | 1 234 matching sessions | 3 `read_sessions` requests (offset 0, 500, 1000); all sessions used; `session_count` 1 234 |
| ST-FLW-03 | No sessions | Filter matches nothing | `groups` [], `session_count` 0 |
| ST-FLW-04 | Unknown ids | `shooter_ids` [999] | `groups` []; no error |
| ST-FLW-05 | database_handler silent | `read_sessions` doesn't answer | `timeout` |
| ST-FLW-06 | Whole result | Fixture `testdata/sessions/statistics_set.json` with a real `database_handler`; every `group_by` × every `period` | Equal to `statistics_set.expected.json` (checked by hand once) |
| ST-FLW-07 | Retired records | Fixture with a retired shooter; `include_retired` false, then true | Their sessions left out, then included |

## Performance (performance)
What is tested: the size the design names ("a few seconds"). *(limit set here)*

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| ST-PRF-01 | Large database | 10 000 sessions with 60 shots each, real `database_handler`; `group_by` `shooter`, `period` `week` | Answer within 10 s (timeout 90 s) |
