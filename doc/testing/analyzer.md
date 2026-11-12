# Tests: analyzer

Tests of the program `analyzer`.

Design: [analyzer.md](../programs/analyzer.md), [interfaces.md](../interfaces.md#analyzer). Reference data and tools: [overview.md](overview.md).

## Setup
- **Unit tests** call `shot_score` and `analyze` from `scoring.rs` directly.
- **Integration tests** run `analyzer` as a process with `RedisTestServer` and send `analyze_shots` with `TestBus`.
- All cases use `TT-REF` and `DEV-REF` (bullet radius b = 2.0) unless they say otherwise.

## Shot score (unit)
What is tested: the edge rule `score(d) = s_i for the smallest i with d − b ≤ r_i` with a tolerance of 1e-9 mm ([analyzer.md](../programs/analyzer.md#shot-score-edge-rule)).

| ID | Case | Distance `d` of the shot center (mm) | Expected score |
|---|---|---|---|
| AN-SCR-01 | Center | 0 | 12 |
| AN-SCR-02 | Inside the white ring | 6.9 | 12 |
| AN-SCR-03 | Exactly on the line | 7 | 12 |
| AN-SCR-04 | Within the tolerance | 7 + 5e-10 | 12 |
| AN-SCR-05 | Just outside | 7.001 | 11 |
| AN-SCR-06 | Rings 11 to 7 on their lines | 12, 17, 22, 27, 32 | 11, 10, 9, 8, 7 |
| AN-SCR-07 | Just outside the black | 32.001 | 6 |
| AN-SCR-08 | Outermost ring on its line | 92 | 1 |
| AN-SCR-09 | Miss | 92.001; 500 | 0; 0 |
| AN-SCR-10 | Without bullet radius | `shot_score` with bullet radius 0: 5; 5.001 | 12; 11 |

## Session analysis (unit)
What is tested: `analyze`: shot scores, score, grouping score, grouping shift, mean point ([analyzer.md](../programs/analyzer.md#definitions)).

| ID | Case | Shots (`angle_turns`, `radius_mm`) | Expected outcome |
|---|---|---|---|
| AN-ANL-01 | No shots | none | `shot_scores` [], `score` 0, `grouping_score` 0, `grouping_shift` (0, 0), `mean_point` null |
| AN-ANL-02 | `S-REF` | A–E of `S-REF` | `shot_scores` [12, 11, 9, 6, 0], `score` 38, `mean_point` (7.435, 9.035) within 1e-3 |
| AN-ANL-03 | One shot off center | (0.25, 50) | `score` 5; `grouping_score` 12; `grouping_shift` (43.0, 0.0) ±0.05 (the smallest shift that brings the shot into the white: 50 − 7) |
| AN-ANL-04 | Three shots on one spot | (0.25, 50) three times | `score` 15; `grouping_score` 36; `grouping_shift` (43.0, 0.0) ±0.05 |
| AN-ANL-05 | Tie: no shift is better | (0.25, 10) and (0.75, 10) | `shot_scores` [11, 11]; `score` 22; `grouping_score` 22; `grouping_shift` (0, 0) (smallest shift wins the tie) |
| AN-ANL-06 | Mean point | (0, 10) and (0.25, 10) | `mean_point` (5, 5) |
| AN-ANL-07 | Grouping never worse | 10 000 random sessions with 1–60 shots (`proptest`) | `grouping_score >= score` every time |
| AN-ANL-08 | Same result every time | The same input 100 times | Identical output (bit-equal) |
| AN-ANL-09 | `rings` in the request are ignored | `TT-REF` with 3 made-up rings | Same result as with the generated rings |
| AN-ANL-10 | Compared with an exact reference | 1 000 random sessions with 2–10 shots within 40 mm of the center; reference: `S(c)` at (0, 0), at every shot, and at every intersection point of two circles of radius `r_i + b` around the shots (the maximum of `S` is reached at one of these points, because `S` only changes on these circles) | `grouping_score` never above the reference, and equal in at least 99 % of the sessions; the miss rate is written to the test report |

## Requests (integration)
What is tested: `analyze_shots` over Redis and its validation ([analyzer.md](../programs/analyzer.md#interface)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| AN-REQ-01 | Valid request | `analyze_shots` with `S-REF`, `TT-REF`, 4.0 | `analyze_shots_response` equal to AN-ANL-02 |
| AN-REQ-02 | Invalid bullet diameter | `bullet_diameter_mm` 0 | `validation_failed` |
| AN-REQ-03 | Invalid target type | Each input of [LIB-TGT-07](shared_libraries.md#ring-generation-and-target-type-validation-unit) | `validation_failed` |
| AN-REQ-04 | Invalid shot | `angle_turns` 1.0; `radius_mm` −1 | `validation_failed` |
| AN-REQ-05 | Several requests at once | 8 worst-case requests at the same time | All answered correctly; the serve loop keeps answering `ping` meanwhile |

## Performance (performance)
What is tested: the limits of the design ([analyzer.md](../programs/analyzer.md#interface)). Release build on the development PC.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| AN-PRF-01 | Typical session | 60 shots within 15 mm of the center | Answer within 200 ms |
| AN-PRF-02 | Worst case | 60 shots spread over the whole target (up to 90 mm from the center) | Answer within 1 s |
