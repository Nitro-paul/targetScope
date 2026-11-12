//! Messages of `camera_reader` (doc/interfaces.md#camera_reader).
//! Source of truth: `schemas/camera_reader/`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::Request;
use crate::model::CameraControl;

pub const STREAM: &str = "camera_reader:requests";

/// Requests understood by `camera_reader`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum CameraReaderRequest {
    Ping {},
    CameraStatus(CameraStatus),
    CameraListDevices(CameraListDevices),
    CameraListControls(CameraListControls),
    CameraSetControl(CameraSetControl),
}

/// Events on `camera_reader:events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum CameraReaderEvent {
    CameraStateChanged(CameraStateChanged),
}

/// `camera_status` request (doc/interfaces.md#camera_status, schemas/camera_reader/camera_status.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraStatus {}

/// `camera_status_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraStatusResponse {
    // TODO: fields
}

impl Request for CameraStatus {
    type Response = CameraStatusResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `camera_list_devices` request (doc/interfaces.md#camera_list_devices, schemas/camera_reader/camera_list_devices.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraListDevices {}

/// `camera_list_devices_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraListDevicesResponse {
    // TODO: fields
}

impl Request for CameraListDevices {
    type Response = CameraListDevicesResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `camera_list_controls` request (doc/interfaces.md#camera_list_controls, schemas/camera_reader/camera_list_controls.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraListControls {}

/// `camera_list_controls_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraListControlsResponse {
    pub controls: Vec<CameraControl>,
}

impl Request for CameraListControls {
    type Response = CameraListControlsResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `camera_set_control` request (doc/interfaces.md#camera_set_control, schemas/camera_reader/camera_set_control.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraSetControl {
    pub id: i64,
    pub value: i64,
}

/// `camera_set_control_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraSetControlResponse {
    pub controls: Vec<CameraControl>,
}

impl Request for CameraSetControl {
    type Response = CameraSetControlResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `camera_state_changed` event (doc/interfaces.md#camera_state_changed, schemas/camera_reader/camera_state_changed.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraStateChanged {
    // TODO: fields
}
