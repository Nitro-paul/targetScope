// doc/interfaces.md#camera_reader

use serde::{Deserialize, Serialize};

use super::{request, Ping};
use crate::model::CameraControl;

pub const PROGRAM: &str = "camera_reader";
pub const STREAM: &str = "camera_reader:requests";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum CameraReaderRequest {
    Ping(Ping),
    CameraStatus(CameraStatus),
    CameraListDevices(CameraListDevices),
    CameraListControls(CameraListControls),
    CameraSetControl(CameraSetControl),
}

// doc/interfaces.md#events-camera_readerevents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum CameraReaderEvent {
    CameraStateChanged(CameraStateChanged),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CameraState {
    Connected,
    Disconnected,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CameraSource {
    V4l2,
    Rtsp,
    None,
}

// doc/interfaces.md#camera_status
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CameraStatus {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraStatusResponse {
    pub state: CameraState,
    pub camera_source: CameraSource,
    pub device: Option<String>,
    pub camera_key: Option<String>,
    pub width_px: Option<i64>,
    pub height_px: Option<i64>,
    pub fps: Option<f64>,
    pub frames_published: i64,
}
request!(CameraStatus, CameraStatusResponse, "camera_status", 5);

// doc/interfaces.md#camera_list_devices
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CameraListDevices {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceFormat {
    pub width_px: i64,
    pub height_px: i64,
    pub fps: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureDevice {
    pub path: String,
    pub name: String,
    pub bus_info: String,
    pub formats: Vec<DeviceFormat>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraListDevicesResponse {
    pub devices: Vec<CaptureDevice>,
}
request!(CameraListDevices, CameraListDevicesResponse, "camera_list_devices", 5);

// doc/interfaces.md#camera_list_controls
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CameraListControls {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlsResponse {
    pub controls: Vec<CameraControl>,
}
request!(CameraListControls, ControlsResponse, "camera_list_controls", 5);

// doc/interfaces.md#camera_set_control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraSetControl {
    pub id: i64,
    pub value: i64,
}
request!(CameraSetControl, ControlsResponse, "camera_set_control", 5);

// doc/interfaces.md#camera_state_changed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraStateChanged {
    pub state: CameraState,
    pub message: String,
    pub camera_key: Option<String>,
    pub width_px: Option<i64>,
    pub height_px: Option<i64>,
}
