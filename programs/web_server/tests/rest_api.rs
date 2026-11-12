// WS-API: REST API mapping (integration)
// Cases: ../../../doc/testing/web_server.md#rest-api-mapping-integration

/// WS-API-01: `GET /api/session`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_01_get_api_session() {
    todo!("WS-API-01")
}

/// WS-API-02: `POST /api/session/setup` `{ "input_mode": "camera" }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_02_post_api_session_setup_input_mode_camera() {
    todo!("WS-API-02")
}

/// WS-API-03: `PATCH /api/session/setup` `{ "device_id": 1, "target_type_id": 1 }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_03_patch_api_session_setup_device_id_1_target_type() {
    todo!("WS-API-03")
}

/// WS-API-04: `PUT /api/session/notes` `{ "notes": "x" }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_04_put_api_session_notes_notes_x() {
    todo!("WS-API-04")
}

/// WS-API-05: `POST /api/session/new-paper`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_05_post_api_session_new_paper() {
    todo!("WS-API-05")
}

/// WS-API-06: `POST /api/session/calibrate` with Corners
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_06_post_api_session_calibrate_with_corners() {
    todo!("WS-API-06")
}

/// WS-API-07: `POST /api/session/start`, `/pause`, `/resume`, `/stop`, `/save`, `/cancel`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_07_post_api_session_start_pause_resume_stop_save_ca() {
    todo!("WS-API-07")
}

/// WS-API-08: `POST /api/session/shots` `{ "angle_turns": 0.25, "radius_mm": 12 }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_08_post_api_session_shots_angle_turns_0_25_radius_m() {
    todo!("WS-API-08")
}

/// WS-API-09: `PUT /api/session/shots/3` `{ "angle_turns": 0.5, "radius_mm": 20 }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_09_put_api_session_shots_3_angle_turns_0_5_radius_m() {
    todo!("WS-API-09")
}

/// WS-API-10: `DELETE /api/session/shots/3`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_10_delete_api_session_shots_3() {
    todo!("WS-API-10")
}

/// WS-API-11: `POST /api/session/photo` multipart (JPEG and Corners)
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_11_post_api_session_photo_multipart_jpeg_and_corner() {
    todo!("WS-API-11")
}

/// WS-API-12: `POST /api/session/photo` JSON `{ "staged_file": "upload_….jpg", "corners": … }` (retry)
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_12_post_api_session_photo_json_staged_file_upload_j() {
    todo!("WS-API-12")
}

/// WS-API-13: `POST /api/sessions/query` `{ filter, order_by, limit, offset }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_13_post_api_sessions_query_filter_order_by_limit_of() {
    todo!("WS-API-13")
}

/// WS-API-14: `GET /api/sessions/7`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_14_get_api_sessions_7() {
    todo!("WS-API-14")
}

/// WS-API-15: `DELETE /api/sessions/7`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_15_delete_api_sessions_7() {
    todo!("WS-API-15")
}

/// WS-API-16: `GET`, `POST /api/shooters`; `PUT`, `DELETE /api/shooters/4`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_16_get_post_api_shooters_put_delete_api_shooters_4() {
    todo!("WS-API-16")
}

/// WS-API-17: The same four for `/api/devices`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_17_the_same_four_for_api_devices() {
    todo!("WS-API-17")
}

/// WS-API-18: `PUT /api/devices/3/image` multipart
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_18_put_api_devices_3_image_multipart() {
    todo!("WS-API-18")
}

/// WS-API-19: `DELETE /api/devices/3/image`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_19_delete_api_devices_3_image() {
    todo!("WS-API-19")
}

/// WS-API-20: The same four for `/api/target-types`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_20_the_same_four_for_api_target_types() {
    todo!("WS-API-20")
}

/// WS-API-21: `POST /api/target-types/photo` multipart
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_21_post_api_target_types_photo_multipart() {
    todo!("WS-API-21")
}

/// WS-API-22: `POST /api/target-types/photo/capture`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_22_post_api_target_types_photo_capture() {
    todo!("WS-API-22")
}

/// WS-API-23: `POST /api/target-types/photo/prepare`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_23_post_api_target_types_photo_prepare() {
    todo!("WS-API-23")
}

/// WS-API-24: `GET /api/camera`, `/api/camera/devices`, `/api/camera/controls`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_24_get_api_camera_api_camera_devices_api_camera_con() {
    todo!("WS-API-24")
}

/// WS-API-25: `PUT /api/camera/controls/9963776` `{ "value": 5 }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_25_put_api_camera_controls_9963776_value_5() {
    todo!("WS-API-25")
}

/// WS-API-26: `GET /api/camera/lens`; `POST …/start`, `/capture`, `/finish`, `/cancel`; `DELETE /api/camera/lens`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_26_get_api_camera_lens_post_start_capture_finish_ca() {
    todo!("WS-API-26")
}

/// WS-API-27: `GET /api/calibration`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_27_get_api_calibration() {
    todo!("WS-API-27")
}

/// WS-API-28: `GET /api/settings`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_28_get_api_settings() {
    todo!("WS-API-28")
}

/// WS-API-29: `PUT /api/settings/camera_fps` `{ "value": "15", "active": true }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_29_put_api_settings_camera_fps_value_15_active_true() {
    todo!("WS-API-29")
}

/// WS-API-30: `POST /api/settings/reset` `{ "names": ["camera_fps"], "move_images": false }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_30_post_api_settings_reset_names_camera_fps_move_im() {
    todo!("WS-API-30")
}

/// WS-API-31: `PUT /api/settings/image_dir` `{ "path": "/srv/img", "move_images": true }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_31_put_api_settings_image_dir_path_srv_img_move_ima() {
    todo!("WS-API-31")
}

/// WS-API-32: `POST /api/statistics` `{ filter, group_by, period }`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_32_post_api_statistics_filter_group_by_period() {
    todo!("WS-API-32")
}

/// WS-API-33: `POST /api/analyze`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_33_post_api_analyze() {
    todo!("WS-API-33")
}

/// WS-API-34: `POST /api/exports`; `GET /api/exports`; `DELETE /api/exports/a.csv`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_34_post_api_exports_get_api_exports_delete_api_expo() {
    todo!("WS-API-34")
}

/// WS-API-35: `POST /api/imports` multipart `.zip`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_35_post_api_imports_multipart_zip() {
    todo!("WS-API-35")
}

/// WS-API-36: `GET /api/system`; `POST /api/system/programs/analyzer/restart`
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_36_get_api_system_post_api_system_programs_analyzer() {
    todo!("WS-API-36")
}

/// WS-API-37: Request timeouts
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_37_request_timeouts() {
    todo!("WS-API-37")
}

/// WS-API-38: No way to change a saved session
#[tokio::test]
#[ignore = "not implemented"]
async fn ws_api_38_no_way_to_change_a_saved_session() {
    todo!("WS-API-38")
}
