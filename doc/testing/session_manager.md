# Tests: session_manager

Tests of the program `session_manager`.

Design: [session_manager.md](../programs/session_manager.md), [interfaces.md](../interfaces.md#session_manager). Reference data and tools: [overview.md](overview.md).

## Setup
- **Actor tests** (integration) run `session_manager` as a process with `RedisTestServer` (for its state and idempotency keys), `FakeClock`, and `FakeProgram`s as `image_processor`, `analyzer` and `database_handler`. Events are published into `image_processor:events` and `database_handler:events` by the test; `session_manager:events` is checked with `EventProbe`.
- **Default answers of the fakes** (a case may change them):
  - `database_handler`: `read_devices` → `DEV-REF`, `read_target_types` → `TT-REF`, `create_session` → `{ id: 1, uuid, images }`;
  - `image_processor`: `live_start` → `calibrated`; `live_set_counting`, `live_stop`, `live_uncount_hole` → `{}`; `capture_target_image` → a new `target_<n>.jpg` that exists in the staging folder; `live_new_paper` → `changed` true with the next `live_paper` and a new `target_<n>.jpg`; `detect_shots_in_photo` → 3 shots and a `photo_<n>.jpg`;
  - `analyzer`: answers with the result of the real `analyze` function.
- "In camera / manual / photo mode" means a session set up with that `input_mode`. "Ready" means device `DEV-REF` and target type `TT-REF` are set.

## State machine (integration)
What is tested: every request in every state and input mode ([session_manager.md](../programs/session_manager.md#state-machine)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-STM-01 | Complete matrix | For every reachable state, every input mode and every request: bring the session into the state, send the request with a valid payload | Result as in the matrix below |
| SM-STM-02 | Refused requests change nothing | Every ✗ cell of the matrix | `invalid_state`; snapshot and Redis key unchanged; no request to another program; no `session_changed` |
| SM-STM-03 | Every change is published | Every allowed request that changes the session | Exactly one `session_changed` with the new snapshot (plus one per later analysis result) |

Matrix: `→ x` = allowed, new state `x`; `✓` = allowed, state stays; `✗` = `invalid_state`. Letters restrict a cell to input modes: C camera, M manual, P photo; in the other modes the cell is ✗. Photo sessions never reach `running` or `paused`.

| Request ↓ / state → | idle | setup | running | paused | review | recovered |
|---|---|---|---|---|---|---|
| `session_get` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `session_setup` | → setup | ✗ | ✗ | ✗ | ✗ | ✗ |
| `session_update_setup` | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ |
| `session_calibrate` | ✗ | ✓ C | ✗ | ✓ C | ✗ | ✗ |
| `session_start` | ✗ | → running C M | ✗ | ✗ | ✗ | ✗ |
| `session_upload_photo` | ✗ | → review P | ✗ | ✗ | ✓ P | ✗ |
| `session_pause` | ✗ | ✗ | → paused | ✗ | ✗ | ✗ |
| `session_resume` | ✗ | ✗ | ✗ | → running | ✗ | ✗ |
| `session_add_shot` | ✗ | ✗ | ✓ M | ✗ | ✓ | ✗ |
| `session_update_shot` | ✗ | ✗ | ✓ M | ✗ | ✓ | ✗ |
| `session_delete_shot` | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ |
| `session_new_paper` | ✗ | ✗ | ✓ C M | ✓ C M | ✗ | ✗ |
| `session_set_notes` | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `session_stop` | ✗ | ✗ | → review | → review | ✗ | ✗ |
| `session_save` | ✗ | ✗ | ✗ | ✗ | → idle | → idle |
| `session_cancel` | ✗ | → idle | → idle | → idle | → idle | → idle |

## Guards (integration)
What is tested: the error codes of failed guards ([session_manager.md](../programs/session_manager.md#transitions-and-actions)): missing or invalid values → `validation_failed`; a resource that isn't ready → `unavailable`.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-GRD-01 | Start without device | Manual mode, target type set, no device; `session_start` | `validation_failed` naming `device_id` |
| SM-GRD-02 | Start without target type | Device set, no target type | `validation_failed` naming `target_type_id` |
| SM-GRD-03 | Start before calibration | Camera mode, ready, `live_start` answered `not_calibrated` | `unavailable` |
| SM-GRD-04 | Resume before calibration | Camera mode, paused; `calibration_changed` `lost`; `session_resume` | `unavailable`; stays paused |
| SM-GRD-05 | Changing the input mode | `session_update_setup` with `input_mode` | `validation_failed` |
| SM-GRD-06 | Retired records | `session_update_setup` with a retired shooter, device or target type | `validation_failed` |
| SM-GRD-07 | Unknown records | `session_update_setup` with an unknown `device_id` (fake returns nothing) | `validation_failed` |
| SM-GRD-08 | Invalid shot | `session_add_shot` with `angle_turns` 1.0; with `paper_no` 0; with `paper_no` above the current paper | `validation_failed` |
| SM-GRD-09 | Invalid index | `session_update_shot` and `session_delete_shot` with index −1 and index = number of shots | `validation_failed` |
| SM-GRD-10 | Notes length | `session_set_notes` with 10 001 characters; with 10 000 | `validation_failed`; ok |
| SM-GRD-11 | Save without shots | Review without shots; `session_save` | `validation_failed` |
| SM-GRD-12 | Save without device | Recovered session whose device was deleted; `session_save` | `validation_failed` naming `device_id` |
| SM-GRD-13 | Calibrate before live mode | Camera mode, device not set yet; `session_calibrate` | `unavailable` |
| SM-GRD-14 | Photo before ready | Photo mode without target type; `session_upload_photo` | `validation_failed` |

## Setup (integration)
What is tested: `session_setup`, `session_update_setup` and starting live mode.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-SET-01 | New session | `session_setup` with `input_mode` `camera` | State `setup`; `paper_no` 1; `shots` []; `duration_s` 0; `analysis` null; `session_changed`; Redis key `session_manager:current` written |
| SM-SET-02 | Live mode starts | Camera mode; `session_update_setup` with device and target type | `read_devices` and `read_target_types` sent; `live_start` with `TT-REF` and `bullet_diameter_mm` 4.0; `calibration` in the snapshot from the response |
| SM-SET-03 | Other fields don't restart live mode | Then `session_update_setup` with `distance_m` only | No `live_start` |
| SM-SET-04 | Device change restarts live mode | Then another device (bullet 4.5) | `live_start` again with 4.5 |
| SM-SET-05 | No live mode in manual and photo mode | Manual and photo mode, ready | No `live_start` |
| SM-SET-06 | Without shooter | `shooter_id` null | Ok |
| SM-SET-07 | Notes in setup | `session_set_notes` | `notes` set in the snapshot |

## Camera sessions (integration)
What is tested: the actions in camera mode.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-CAM-01 | Start | Ready and calibrated; `session_start` | `started_at` = now; `live_set_counting(true)`; state `running` |
| SM-CAM-02 | Running time | Run 60 s, pause 30 s, resume, run 20 s, stop (`FakeClock`) | `duration_s` 80 |
| SM-CAM-03 | Pause and resume | `session_pause`; `session_resume` | `live_set_counting(false)`; `live_set_counting(true)` |
| SM-CAM-04 | Stop | `session_stop` | `capture_target_image`, then `live_stop`, in this order; the image is in `paper_images` for the current paper; state `review` |
| SM-CAM-05 | Stop when the image fails | `capture_target_image` answers `unavailable` | State `review` anyway; `live_stop` sent; at saving the response contains a warning that the paper has no image |
| SM-CAM-06 | Cancel | `session_cancel` while running | `live_stop`; Redis key deleted; state `idle`; `session_changed` |
| SM-CAM-07 | Manual calibration | `session_calibrate` in setup and in paused | `live_calibrate` with the corners |
| SM-CAM-08 | Delete a camera shot | Shot with `hole_id` 7; `session_delete_shot` in running, in paused, in review | `live_uncount_hole(7)` in running and paused; nothing sent in review (live mode ended) |

## Papers (integration)
What is tested: paper changes and paper numbers ([session_manager.md](../programs/session_manager.md#target-papers)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-PAP-01 | New paper, camera | Running on paper 1; `session_new_paper` | `live_new_paper` with `from_live_paper` 1; response `changed` true → its image in `paper_images` for paper 1; `paper_no` 2 |
| SM-PAP-02 | Already changed automatically | `live_new_paper` answers `changed` false, `live_paper` 2; later a `paper_replaced` event with `live_paper` 2 arrives | `paper_no` stays 1 until the event; then the event's image for paper 1 and `paper_no` 2; exactly one change |
| SM-PAP-03 | New paper, manual | Manual mode; `session_new_paper` | `paper_no` + 1; no request to `image_processor`; no image |
| SM-PAP-04 | Shots after a change | Paper changed to 2; `shot_detected` with `live_paper` 2 | The shot gets `paper_no` 2 |
| SM-PAP-05 | Late shot of the old paper | Paper changed to 2; then `shot_detected` with `live_paper` 1 | The shot gets `paper_no` 1 |
| SM-PAP-06 | Offset after a live mode restart | Running on paper 3; the target type changes (DB event) → `live_start`; then `shot_detected` with `live_paper` 1 | The shot gets `paper_no` 3 |
| SM-PAP-07 | Paper without shots | Change the paper twice without shots in between; stop; save | `paper_count` 3; no shots on paper 2; saved |

## Photo sessions (integration)
What is tested: shots from photos.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-PHO-01 | First photo | Photo mode, ready; `session_upload_photo` | `detect_shots_in_photo` with `staged_file`, `TT-REF`, 4.0 and the corners if given; 3 shots with origin `photo`, `paper_no` 1, `confidence`; `paper_images` `[{ 1, photo_1.jpg }]`; state `review`; analysis requested |
| SM-PHO-02 | Second photo | In review; `session_upload_photo` again | 3 more shots on paper 2; second image for paper 2 |
| SM-PHO-03 | Paper not found | `detect_shots_in_photo` answers `processing_failed` | Error passed on; state stays `setup`; no shots |
| SM-PHO-04 | Save | Save a photo session | `duration_s` null; `images` = the photos |

## Manual sessions (integration)
What is tested: shots entered by hand.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-MAN-01 | Start | Manual mode, ready; `session_start` | `running`; no request to `image_processor` |
| SM-MAN-02 | Add a shot | `session_add_shot` (0.25, 12) | Shot with origin `manual`, current `paper_no`, `confidence` null, `hole_id` null; analysis requested |
| SM-MAN-03 | Change and delete | `session_update_shot`, `session_delete_shot` | Position changed; shot removed; analysis after each |
| SM-MAN-04 | Add in review | Two papers; in review `session_add_shot` without `paper_no`; then with `paper_no` 1 | First on paper 2 (the last), second on paper 1 |
| SM-MAN-05 | Save | Save a manual session | `images` [] |

## Events (integration)
What is tested: handling of `image_processor` and `database_handler` events ([session_manager.md](../programs/session_manager.md#shot_detected-events)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-EVT-01 | Shot while running | Camera mode, running; `shot_detected` (`hole_id` 3, `live_paper` 1, `confidence` 0.9) | Shot appended with origin `camera`, `hole_id` 3, `confidence` 0.9, `paper_no` 1; `session_changed`; analysis requested |
| SM-EVT-02 | Same event twice | The same `event_id` again | Added once |
| SM-EVT-03 | Shot while paused | Paused; `shot_detected` | Not added; `live_uncount_hole` with its `hole_id` |
| SM-EVT-04 | Shot in setup | Setup (live mode active); `shot_detected` | Not added; `live_uncount_hole` |
| SM-EVT-05 | Shot in review | Review (live mode ended); `shot_detected` | Not added; nothing sent |
| SM-EVT-06 | Paper replaced | Running; `paper_replaced` with `live_paper` 2 | Its image in `paper_images` for paper 1; `paper_no` 2 |
| SM-EVT-07 | Paper replaced twice | The same event again; then an event with `live_paper` 2 again but a new `event_id` | One change only |
| SM-EVT-08 | Paper replaced outside running and paused | `paper_replaced` in setup and in review | Ignored |
| SM-EVT-09 | Calibration changes | `calibration_changed` (`lost`) | Snapshot `calibration` updated; `session_changed` |
| SM-EVT-10 | Device changed during setup | Camera mode, setup; `data_changed` for the device; the fake now returns bullet 4.5 | Device read again; analysis again; `live_start` with 4.5 |
| SM-EVT-11 | Device changed during review | Review; same event | Read again; analysis again; no `live_start` |
| SM-EVT-12 | Target type changed while running | Camera mode, running, 3 camera shots; `data_changed` for the target type | `live_start` again; the 3 shots get origin `manual` and `hole_id` null; `live_paper_offset` = `paper_no` − 1 |
| SM-EVT-13 | Device deleted | `data_changed` (devices, deleted) for the session's device | `device_id` cleared in the setup; `session_save` later answers `validation_failed` |
| SM-EVT-14 | Other device changed | `data_changed` for another device | Nothing happens |
| SM-EVT-15 | Reload everything | `data_changed` with `table: "all"` | Device and target type read again |
| SM-EVT-16 | Events while session_manager was down | Running; stop the program; 3 `shot_detected` are published; start it | State `recovered`; the 3 events are read from the consumer group, acknowledged and ignored |

## Live analysis (integration)
What is tested: scoring after every shot change ([session_manager.md](../programs/session_manager.md#live-analysis)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-ANL-01 | After every change | Add a shot | `analyze_shots` with all shot positions, `TT-REF` and 4.0; result in `analysis`; `analysis_outdated` false afterwards |
| SM-ANL-02 | Burst of shots | The analyzer takes 500 ms; 10 `shot_detected` within 100 ms | 2 analyses in total (the running one and one more); final `analysis` covers all 10 shots |
| SM-ANL-03 | Outdated result | A result arrives for an older shot list | Dropped; `analysis` unchanged |
| SM-ANL-04 | Analyzer not answering | The analyzer times out | `analysis_outdated` true; the last result kept; retried at the next change |
| SM-ANL-05 | Published | Each accepted result | One `session_changed` |

## Saving (integration)
What is tested: `session_save` ([session_manager.md](../programs/session_manager.md#saving)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-SAV-01 | Camera session | Camera session with 2 papers, 5 shots, notes; `session_save` | `create_session` with: `shooter_id`, `device_id`, `target_type_id`, `distance_m`, `single_handed`, `supported`, shots in order with positions and `paper_no`, `shot_count` 5, `paper_count` 2, `score` and `grouping_score` from the analysis, `duration_s`, `notes`, `images` = `paper_images`, `request_id` = `save_request_id`. Response has `session_id`; state `idle`; Redis key deleted; `session_saved` and `session_changed` |
| SM-SAV-02 | Id stored before sending | `sm.after_create_session_sent` stops; read the Redis key | `save_request_id` is already in the stored state |
| SM-SAV-03 | Records read again | The fake now returns bullet 4.5 for the device | Device read again; new analysis; `create_session` carries the new scores |
| SM-SAV-04 | Analyzer down | Analysis outdated and the analyzer doesn't answer | `unavailable`; state unchanged; no `create_session` |
| SM-SAV-05 | Image expired | `create_session` answers `not_found` for a staged image | Sent again without the missing image, with the same `request_id`; the response has a warning naming the paper |
| SM-SAV-06 | Database error | `create_session` answers `database_error` | Error returned; state `review`; a second `session_save` uses the same `save_request_id` |
| SM-SAV-07 | Crash after sending | `sm.after_create_session_sent` aborts the process; restart; `session_save` | State `recovered`; `create_session` with the same `request_id`; exactly one session in the fake's records |
| SM-SAV-08 | Warning from the stop | Image at `session_stop` failed (SM-CAM-05) | `warnings` contains it |
| SM-SAV-09 | Save a recovered session | `session_save` in `recovered` | Same as from review |

## Persistence and recovery (integration)
What is tested: the state in Redis and the restart ([session_manager.md](../programs/session_manager.md#persistence-and-recovery)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-REC-01 | State stored | After each request of a full camera session | `session_manager:current` equals the snapshot plus the internal fields (`save_request_id`, `running_since_ms`, cached device and target type, `shots_version`, `live_paper_offset`, last `live_paper`, warnings) |
| SM-REC-02 | Stored with the idempotency record | Watch Redis with `MONITOR` during a request | `SET session_manager:current` and `SET session_manager:processed:<id>` in one `MULTI`/`EXEC` |
| SM-REC-03 | Failed write | `sm.before_exec` returns an error during `session_add_shot` | Error; `session_get` still shows the old state; no `session_changed` |
| SM-REC-04 | Start without state | No key; start | `idle` |
| SM-REC-05 | Start with an idle state | Key with `state` `idle` | `idle` |
| SM-REC-06 | Start after a crash while running | Running camera session, last change at t = 100 s; restart at t = 500 s | `recovered`, `recovered_from` `running`; running time counted up to t = 100 s; `live_stop` sent; device and target type read again; analysis (retried every 5 s until the analyzer answers); `session_changed` |
| SM-REC-07 | Images after recovery | Recovered while running on paper 2 (paper 1 changed with image) | Saving keeps the image of paper 1; paper 2 has none |
| SM-REC-08 | Recovered without shots | Recovered session without shots | `session_save` → `validation_failed`; `session_cancel` → `idle` |
| SM-REC-09 | Redis restart | Stop the Redis test server and start it on the same folder | `session_manager:current` still there (AOF) |
| SM-REC-10 | Recovered from review | Restart while in review | `recovered_from` `review`; all paper images kept |

## Duplicate protection (integration)
What is tested: repeated requests and events ([interfaces.md](../interfaces.md#duplicate-protection-idempotency)).

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-IDM-01 | Same request twice | `session_add_shot` twice with the same `request_id` | Second response identical; one shot; key `session_manager:processed:<id>` with a TTL of about 24 h |
| SM-IDM-02 | Every changing request | Parametrized over all requests that change the session | Each effect only once |
| SM-IDM-03 | Event delivered again after a crash | Crash at `bus.before_ack` while handling a `shot_detected`; restart | The event isn't applied twice |

## Error handling and timing (integration)
What is tested: slow or missing programs, many requests at once.

| ID | Case | Steps / input | Expected outcome |
|---|---|---|---|
| SM-ERR-01 | `image_processor` silent at stop | `capture_target_image` and `live_stop` don't answer | State `review` within 30 s; warning for the paper |
| SM-ERR-02 | `database_handler` silent at setup | `read_devices` doesn't answer | `timeout`; setup unchanged |
| SM-ERR-03 | Calibration lost while running | `calibration_changed` `lost` | Stays `running`; snapshot shows `lost` |
| SM-ERR-04 | Many requests at once | Manual mode, running; 100 `session_add_shot` at the same time | 100 shots; none lost |
| SM-ERR-05 | Nested timeouts | Every dependency answers after 4 s; `session_save` | Answered before its 30 s timeout |
