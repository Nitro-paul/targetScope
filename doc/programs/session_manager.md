# session_manager

Program `session_manager` (Rust): the state machine of the current shooting session. It collects the shots (from the camera, manual clicks or a photo), has them scored live, lets the user review them, and saves the session as a record. Requests and events: [interfaces.md](../interfaces.md#session_manager).

## Responsibilities
- Holds the one current session and its state; rejects requests that aren't allowed in the current state (`invalid_state`).
- Controls `image_processor`: live mode, counting on/off, manual calibration, uncounting holes, paper changes, photo detection, target image capture.
- Keeps track of the target papers of the session and their images.
- Adds shots from `shot_detected` events while running.
- Has the shots scored by `analyzer` after every shot change, so score and grouping score are shown live.
- Saves the finished session through `database_handler`. After saving, the session is a record and can't be changed.
- Persists the current session in Redis after every change, and offers it for saving or discarding after a restart.
- Sends `session_changed` after every change.

## Technology
`targetscope_common` (bus, idempotency, settings), `tokio`. No database, no image processing.

## Input modes
`session_setup` fixes how shots are entered for the whole session (`input_mode`):

| Input mode | Shots come from | Target image stored with the session |
|---|---|---|
| `camera` | `shot_detected` events of the live mode | one per paper: taken at each paper change, and for the last paper at `session_stop` |
| `manual` | clicks on the SVG target (`session_add_shot`) | none |
| `photo` | `detect_shots_in_photo` on uploaded photos (each photo is one paper) | the transformed photos |

In every input mode, shots can be added, moved and deleted by hand in review.

## Target papers
A session can use several target papers (e.g. air rifle: one shot per card; air pistol: 5 shots per card). `paper_no` starts at 1; every shot gets the current `paper_no`.
- **camera:** `session_new_paper` (button "new paper") calls `live_new_paper`; a `paper_replaced` event (paper swap detected by `image_processor`) does the same. Both deliver the image of the finished paper, which is added to `paper_images`; then `paper_no` increases.

  Live mode numbers its papers itself (`live_paper`, see [live_new_paper](../interfaces.md#live_new_paper)). session_manager maps them with an offset set at every `live_start` (`paper_no = live_paper + live_paper_offset`), and assigns every shot by the `live_paper` in its `shot_detected` event, not by the order in which events arrive. This way a shot that was detected on the old paper but arrives after the paper change still gets the old paper's number, and a paper change that was requested and detected at the same time counts once (`from_live_paper`).
- **manual:** `session_new_paper` only increases `paper_no`.
- **photo:** every `session_upload_photo` adds the photo's shots as a new paper, with the transformed photo as its image.

A paper without shots is allowed (e.g. a sighting card).

## State machine
```
            session_setup             session_start
   idle ───────────────────> setup ───────────────────> running ◄──── session_resume ────┐
    ^                          │                          │  │                           │
    │                          │ session_upload_photo     │  └── session_pause ──> paused
    │                          │ (input mode photo)       │                              │
    │                          v                          │ session_stop                 │ session_stop
    │                        review  <────────────────────┴──────────────────────────────┘
    │                          │  ^
    │                          │  └── add / update / delete shot
    │             session_save │
    └──────── saved <──────────┤
              (record)         │ session_save
                           recovered   (unsaved session found after a restart)

   session_cancel: from setup, running, paused, review and recovered back to idle
```

| State | Meaning | Allowed requests |
|---|---|---|
| `idle` | No session | `session_get`, `session_setup` |
| `setup` | Choosing shooter, device, distance, target type; camera: live view and calibration | `session_update_setup`, `session_calibrate`, `session_start`, `session_upload_photo`, `session_cancel` |
| `running` | Shots are collected and scored live; running time counts | `session_pause`, `session_stop`, `session_new_paper`, `session_add_shot`/`session_update_shot` (manual), `session_delete_shot`, `session_cancel` |
| `paused` | New holes aren't counted; running time stops | `session_resume`, `session_stop`, `session_new_paper`, `session_delete_shot`, `session_calibrate`, `session_cancel` |
| `review` | Shots can be corrected before saving | `session_add_shot`, `session_update_shot`, `session_delete_shot`, `session_upload_photo` (photo), `session_save`, `session_cancel` |
| `recovered` | A session that wasn't saved before a restart | `session_save`, `session_cancel` |

`session_get` is allowed in every state, `session_set_notes` in every state except `idle`.

### Transitions and actions
| Request | Guard | Actions |
|---|---|---|
| `session_setup` | | create empty session with the given input mode |
| `session_update_setup` | values valid; `input_mode` not in the payload; no retired shooter, device or target type | update setup; read device and target type when they change; input mode `camera`: if device and target type are set and one of them changed, `live_start` with the target type and bullet diameter (the live view appears, the paper is searched automatically); `live_paper_offset = paper_no - 1` |
| `session_calibrate` | input mode `camera`, live mode started | `live_calibrate` with the corners |
| `session_start` | input mode `camera` or `manual`; device and target type set; camera: calibrated | `started_at = now`; camera: `live_set_counting(true)` |
| `session_upload_photo` | input mode `photo`; device and target type set | `detect_shots_in_photo`; the shots get origin `photo` and the next `paper_no`; the transformed image is added to `paper_images`; from setup → review |
| `session_new_paper` | input mode `camera` or `manual` | camera: `live_new_paper` with `from_live_paper` = current live paper; if `changed`, add the returned image to `paper_images` and `paper_no + 1` (if not, the paper was changed automatically and the `paper_replaced` event does it); manual: `paper_no + 1` |
| `session_set_notes` | at most 10 000 characters | set `notes` |
| `session_pause` | | add running time; camera: `live_set_counting(false)` (holes made during the pause are shown gray and not counted) |
| `session_resume` | camera: calibrated | camera: `live_set_counting(true)` |
| `session_stop` | | add running time; camera: `capture_target_image` for the current paper (added to `paper_images`; if it fails, a warning is kept for saving), then `live_stop`; → review |
| `session_add_shot`, `session_update_shot` | index valid, Shot valid, `paper_no` within `1 ..= paper_no` | change shots; added shots get origin `manual` and the given paper (default: current paper) |
| `session_delete_shot` | index valid | remove the shot; if it has a `hole_id` and live mode is still active (running, paused): `live_uncount_hole` |
| `session_save` | at least one shot; device and target type set; current analysis | see [Saving](#saving) |
| `session_cancel` | | camera: `live_stop`; delete state; → idle |

**Failed guards:** a request that isn't allowed in the state or input mode answers `invalid_state`. A request that is allowed but whose guard fails answers `validation_failed` if a value is missing or invalid (no device or target type, no shots at saving, a retired or unknown record, an index out of range, notes too long), and `unavailable` if a resource isn't ready (not calibrated at `session_start` or `session_resume`, live mode not started yet at `session_calibrate`, the analyzer not answering at `session_save`). In every case nothing changes.

After every change of the shots (detected, added, changed, deleted, or from a photo), the shots are analyzed again (see [Live analysis](#live-analysis)).

Device and target type are read from `database_handler` (`read_devices`, `read_target_types`) when they are set, and kept in the state, so the bullet diameter and anchors are at hand without further requests.

### shot_detected events
Read from `image_processor:events` with consumer group `session_manager` (as are `paper_replaced` events: in states running and paused, if the event's `live_paper` is newer than the last known one, its image is added to `paper_images` and `paper_no` set accordingly; ignored otherwise).
- In state `running` with input mode `camera`: the shot is appended with origin `camera`, its confidence, its `hole_id` and `paper_no = live_paper + live_paper_offset`.
- Otherwise (e.g. the event arrives just after a pause): the event is ignored, and while live mode is still active `live_uncount_hole` is sent, so the live view doesn't highlight a hole that isn't a session shot.

Each `event_id` is recorded with the state change, so an event that is delivered twice adds the shot only once.

### calibration_changed events
In input mode `camera`, the newest CalibrationStatus is copied into the session snapshot (`calibration`), so the UI shows it together with the session.

### data_changed events
Read from `database_handler:events` (plain read). If the session's device or target type is changed or deleted while the session isn't saved yet (they aren't locked until a saved session uses them):
- **changed:** read it again; analyze again; input mode `camera`, live mode active (setup, running, paused) and the target type or bullet diameter changed: `live_start` again (the paper is searched again, holes are scanned again, and all current camera shots get origin `manual`, because their hole ids are no longer known to the new live mode).
- **deleted:** the setup field is cleared; saving is only possible after choosing another one.
- **`all` / `reload`** (sent by `database_handler` after a restart or reconnect): device and target type are read again, handled like **changed**.

Before saving, device and target type are read once more, so a missed event can't lead to a session saved with outdated values.

## Live analysis
Score and grouping score are shown while shooting:
- After every shot change, `analyze_shots` is sent with all shots, the target type and the bullet diameter. The result is stored in `analysis` and sent with `session_changed`.
- Only one analysis runs at a time. If shots change while one is running, one more analysis is done afterwards with the newest shots; results for outdated shot lists are dropped. A burst of shots therefore never queues more than one extra analysis.
- `analysis_outdated` is `true` from a shot change until the matching result arrives. If the analyzer doesn't answer, `analysis` keeps the last result, stays outdated, and is retried on the next change and before saving.

## Saving
1. If not already set: create `save_request_id` (UUID) and persist it in the state. If the program crashes during saving, the retry uses the same id, so `database_handler` doesn't store the session twice.
2. Read device and target type again (see [data_changed events](#data_changed-events)).
3. Make sure `analysis` matches the current shots (analyze again if needed). If the analyzer doesn't answer: the request fails with `unavailable`; nothing is saved without a score.
4. Target images (always the transformed paper, never the raw image): all `paper_images`. In input mode `camera` they were taken at every paper change and at `session_stop`, so the user may already have taken the paper down when saving. Input mode `manual` has none.
5. Send `create_session` with `request_id = save_request_id`: setup values, shots, `shot_count`, `score` and `grouping_score` from the analysis, `duration_s` (running time; `null` for input mode `photo`), `notes`, `paper_count` (= current `paper_no`), every shot's `paper_no`, `images`.
6. If `create_session` fails with `not_found` because a staged image is gone (e.g. a recovered session older than `staging_max_age_hours`): send it again without the missing images and add a warning to the response. The same `save_request_id` can be used, because `database_handler` only records successful requests.
7. On success: delete the state; send `session_saved` and `session_changed` (idle); respond with `session_id`. From now on the session is a record.
8. On any other failure: stay in the current state and return the error. The user can retry or cancel.

## Persistence and recovery
The whole `CurrentSession` plus internal fields (`save_request_id`, `running_since_ms`, cached device and target type, `shots_version`, `live_paper_offset`, last known `live_paper`, warnings for saving) is stored as JSON under the Redis key `session_manager:current`. Every change is written in one `MULTI`/`EXEC` together with the idempotency record of the request or event that caused it. Redis persists it with AOF (see [launcher.md](launcher.md#redis)).

On startup:
1. Read `session_manager:current`. If it's missing or `idle`: start in `idle`.
2. Otherwise: `recovered_from` = stored state, state = `recovered`. If the session was running, the running time is counted until the last persisted change.
3. Camera: send `live_stop` (live mode might still be active in `image_processor`).
4. Read device and target type again; if there are shots: analyze (retry every 5 s until `analyzer` answers).
5. Send `session_changed`. The UI shows a dialog: "An unsaved session from <started_at> with <n> shots was found: Save / Discard". Saving requires at least one shot, a device and a target type; otherwise only Discard is possible. A recovered camera session keeps the images taken before the restart; if it was still running or paused, the last paper has no image, because the camera view may have changed since.

## Internal design
```rust
/// All requests and events are handled by one task, one at a time, so the state never races.
struct SessionActor {
    bus: Bus,
    idem: RedisIdempotencyStore,
    state: StoredSession,              // CurrentSession + internal fields
    analysis: AnalysisRunner,          // at most one running analysis + one pending
}

enum Input {
    Request     { id: Uuid, req: SessionManagerRequest, reply: oneshot::Sender<Result<Value, ServiceError>> },
    ShotEvent   { id: Uuid, event: ImageProcessorEvent, ack: oneshot::Sender<()> },
    DataChanged { event: DataChanged },
    Analysis    { shots_version: u64, result: Result<AnalysisResult, BusError> },
}

impl SessionActor {
    async fn run(mut self, rx: mpsc::Receiver<Input>);
    /// Checks the state table, runs the actions, returns the new state.
    async fn apply(&mut self, req: SessionManagerRequest) -> Result<StoredSession, ServiceError>;
    /// MULTI: SET session_manager:current, idempotency record; EXEC; then publish session_changed.
    async fn commit(&mut self, new: StoredSession, id: Uuid, response: &Value) -> Result<(), BusError>;
}
```
`Bus::serve`, the event readers and the analysis task only forward `Input`s to the actor. `shots_version` increases with every shot change; an analysis result is only used if its version is the current one.

## Error handling
| Situation | Behavior |
|---|---|
| Request not allowed in state | `invalid_state`, nothing changes |
| image_processor or database_handler doesn't answer | Error `timeout`/`unavailable` to the caller, state unchanged |
| Analyzer doesn't answer | Shots are still recorded; analysis stays outdated and is retried; saving waits for a current analysis |
| Calibration lost while running (camera) | Stay in `running`; the UI shows it; holes aren't detected until the paper is found again (or the user pauses and calibrates manually) |
| `capture_target_image` fails at `session_stop` | Review starts anyway; the last paper has no image, and `session_save` reports a warning |
| Redis write fails | The request fails; the in-memory state is not changed (state is only replaced after a successful `EXEC`) |

## Testing
All test cases with their expected outcomes: [testing/session_manager.md](../testing/session_manager.md).
