# statistics

Program `statistics` (Rust): calculates statistics over many saved sessions for the statistics page of the web UI: summaries, comparisons between groups, trends over time, and the distribution of all shots. Request: [interfaces.md](../interfaces.md#statistics).

## Responsibilities
- Selects the sessions with a [SessionFilter](../interfaces.md#sessionfilter) (shooters, devices, device categories, calibers, target types, distance, time range, single-handed, supported, shot count), the same filter the history and export use.
- Splits them by target type (scores of different target types can't be compared) and by the requested `group_by`.
- Calculates the [StatisticsValues](../interfaces.md#statisticsvalues) per group, and per period for the trend.
- Returns the shots of each group for the shot distribution plot.

It only reads data (through `database_handler`) and keeps no state. It doesn't score shots: it uses the scores stored with the sessions, which never change (records).

## Technology
`targetscope_common` (bus, model, geometry), `tokio`, `time` (calendar periods). The calculations are plain Rust in their own module (`calc.rs`), testable without Redis.

## Calculation flow (`compute_statistics`)
1. **Read the sessions** with `read_sessions` (the request's SessionFilter, `include_shots = true`), in pages of 500 ordered by `id` (stable even if sessions are saved meanwhile), until all are read. The database handler applies the whole filter, including the device fields.
2. **Read the devices** of the sessions (`read_devices` with `id in …`), needed for `group_by = device_category`.
3. **Group** by `target_type_id` and the `group_by` key (`shooter_id`, `device_id`, device `category`, `distance_m`, `single_handed`, `supported`). Sessions without a shooter form their own group (`{ "shooter_id": null }`).
4. **Per session** (from its shots, in Cartesian mm around the ring center):
   - mean point of impact `m = mean(p_j)`,
   - extreme spread: largest distance between two shots (`O(n²)`, fine for up to a few hundred shots; 0 for one shot),
   - mean radius: mean of `|p_j - m|`.
5. **Per group** and **per period** (`session`, or calendar `day`/`week`/`month` in local time, weeks starting on Monday): average the session values; `score_min`/`score_max` of the session scores; `score_per_shot_avg = Σ score / Σ shot_count`, `grouping_score_per_shot_avg` likewise (weighted by shots, so long sessions count more).
6. **Shots** for the distribution plot: all shots of the group, newest sessions first, at most 5 000.

Groups are sorted by target type name, then by the group key; trend entries by time.

## Performance
Every request reads the selected sessions again; nothing is cached, so results are always current. 10 000 sessions with 60 shots each (600 000 shots) are read and calculated in a few seconds. If that becomes too slow, the database handler can later get an aggregated read request without changing this interface.

## Error handling
| Situation | Behavior |
|---|---|
| No session matches | `ok` with `groups = []`, `session_count = 0` |
| database_handler doesn't answer | `timeout` / `unavailable` |
| Unknown id in the filter | No error: it simply matches no session |

## Testing
- `calc.rs`: extreme spread and mean radius against hand-calculated examples; one-shot sessions; weighting of per-shot averages.
- Period grouping across month and year boundaries and daylight saving time changes.
- Integration test with a test database: known sessions → expected groups and values.
