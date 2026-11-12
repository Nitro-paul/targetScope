//! Messages of `database_handler` (doc/interfaces.md#database_handler).
//! Source of truth: `schemas/database_handler/`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::Request;
use crate::model::{Device, Session, Setting, Shooter, TargetType};

pub const STREAM: &str = "database_handler:requests";

/// Requests understood by `database_handler`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DatabaseHandlerRequest {
    Ping {},
    CreateSession(CreateSession),
    ReadSessions(ReadSessions),
    DeleteSession(DeleteSession),
    SaveShooter(SaveShooter),
    ReadShooters(ReadShooters),
    DeleteShooter(DeleteShooter),
    SaveDevice(SaveDevice),
    ReadDevices(ReadDevices),
    DeleteDevice(DeleteDevice),
    SetDeviceImage(SetDeviceImage),
    DeleteDeviceImage(DeleteDeviceImage),
    SaveTargetType(SaveTargetType),
    ReadTargetTypes(ReadTargetTypes),
    DeleteTargetType(DeleteTargetType),
    ImportShooter(ImportShooter),
    ImportTargetType(ImportTargetType),
    ImportDevice(ImportDevice),
    ImportSession(ImportSession),
    SaveSetting(SaveSetting),
    ReadSettings(ReadSettings),
    DeleteSetting(DeleteSetting),
    ResetSettings(ResetSettings),
    ChangeImageDir(ChangeImageDir),
}

/// Events on `database_handler:events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DatabaseHandlerEvent {
    DataChanged(DataChanged),
}

/// `create_session` request (doc/interfaces.md#create_session, schemas/database_handler/create_session.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSession {
    // TODO: fields
}

/// `create_session_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    // TODO: fields
}

impl Request for CreateSession {
    type Response = CreateSessionResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `read_sessions` request (doc/interfaces.md#read_sessions, schemas/database_handler/read_sessions.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadSessions {
    // TODO: fields
}

/// `read_sessions_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadSessionsResponse {
    pub sessions: Vec<Session>,
    pub total: i64,
}

impl Request for ReadSessions {
    type Response = ReadSessionsResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `delete_session` request (doc/interfaces.md#delete_session, schemas/database_handler/delete_session.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteSession {
    pub id: i64,
}

/// `delete_session_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteSessionResponse {
    pub id: i64,
}

impl Request for DeleteSession {
    type Response = DeleteSessionResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `save_shooter` request (doc/interfaces.md#save_shooter, schemas/database_handler/save_shooter.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveShooter {
    // TODO: fields
}

/// `save_shooter_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveShooterResponse {
    pub id: i64,
}

impl Request for SaveShooter {
    type Response = SaveShooterResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `read_shooters` request (doc/interfaces.md#read_shooters, schemas/database_handler/read_shooters.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadShooters {
    // TODO: fields
}

/// `read_shooters_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadShootersResponse {
    pub shooters: Vec<Shooter>,
}

impl Request for ReadShooters {
    type Response = ReadShootersResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `delete_shooter` request (doc/interfaces.md#delete_shooter, schemas/database_handler/delete_shooter.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteShooter {
    pub id: i64,
}

/// `delete_shooter_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteShooterResponse {
    pub id: i64,
}

impl Request for DeleteShooter {
    type Response = DeleteShooterResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `save_device` request (doc/interfaces.md#save_device, schemas/database_handler/save_device.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveDevice {
    // TODO: fields
}

/// `save_device_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveDeviceResponse {
    pub id: i64,
}

impl Request for SaveDevice {
    type Response = SaveDeviceResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `read_devices` request (doc/interfaces.md#read_devices, schemas/database_handler/read_devices.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDevices {
    // TODO: fields
}

/// `read_devices_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadDevicesResponse {
    pub devices: Vec<Device>,
}

impl Request for ReadDevices {
    type Response = ReadDevicesResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `delete_device` request (doc/interfaces.md#delete_device, schemas/database_handler/delete_device.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDevice {
    pub id: i64,
}

/// `delete_device_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDeviceResponse {
    pub id: i64,
}

impl Request for DeleteDevice {
    type Response = DeleteDeviceResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `set_device_image` request (doc/interfaces.md#set_device_image, schemas/database_handler/set_device_image.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetDeviceImage {
    pub device_id: i64,
    pub staged_file: String,
}

/// `set_device_image_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetDeviceImageResponse {
    pub image_path: String,
}

impl Request for SetDeviceImage {
    type Response = SetDeviceImageResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `delete_device_image` request (doc/interfaces.md#delete_device_image, schemas/database_handler/delete_device_image.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDeviceImage {
    pub device_id: i64,
}

/// `delete_device_image_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteDeviceImageResponse {
    pub device_id: i64,
}

impl Request for DeleteDeviceImage {
    type Response = DeleteDeviceImageResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `save_target_type` request (doc/interfaces.md#save_target_type, schemas/database_handler/save_target_type.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveTargetType {
    // TODO: fields
}

/// `save_target_type_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveTargetTypeResponse {
    pub id: i64,
}

impl Request for SaveTargetType {
    type Response = SaveTargetTypeResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `read_target_types` request (doc/interfaces.md#read_target_types, schemas/database_handler/read_target_types.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadTargetTypes {
    // TODO: fields
}

/// `read_target_types_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadTargetTypesResponse {
    pub target_types: Vec<TargetType>,
}

impl Request for ReadTargetTypes {
    type Response = ReadTargetTypesResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `delete_target_type` request (doc/interfaces.md#delete_target_type, schemas/database_handler/delete_target_type.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteTargetType {
    pub id: i64,
}

/// `delete_target_type_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteTargetTypeResponse {
    pub id: i64,
}

impl Request for DeleteTargetType {
    type Response = DeleteTargetTypeResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `import_shooter` request (doc/interfaces.md#import_shooter, schemas/database_handler/import_shooter.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportShooter {
    // TODO: fields
}

/// `import_shooter_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportShooterResponse {
    // TODO: fields
}

impl Request for ImportShooter {
    type Response = ImportShooterResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `import_target_type` request (doc/interfaces.md#import_target_type, schemas/database_handler/import_target_type.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportTargetType {
    // TODO: fields
}

/// `import_target_type_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportTargetTypeResponse {
    // TODO: fields
}

impl Request for ImportTargetType {
    type Response = ImportTargetTypeResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `import_device` request (doc/interfaces.md#import_device, schemas/database_handler/import_device.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportDevice {
    // TODO: fields
}

/// `import_device_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportDeviceResponse {
    // TODO: fields
}

impl Request for ImportDevice {
    type Response = ImportDeviceResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `import_session` request (doc/interfaces.md#import_session, schemas/database_handler/import_session.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportSession {
    // TODO: fields
}

/// `import_session_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportSessionResponse {
    // TODO: fields
}

impl Request for ImportSession {
    type Response = ImportSessionResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `save_setting` request (doc/interfaces.md#save_setting, schemas/database_handler/save_setting.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveSetting {
    // TODO: fields
}

/// `save_setting_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveSettingResponse {
    pub name: String,
}

impl Request for SaveSetting {
    type Response = SaveSettingResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `read_settings` request (doc/interfaces.md#read_settings, schemas/database_handler/read_settings.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadSettings {
    // TODO: fields
}

/// `read_settings_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadSettingsResponse {
    pub settings: Vec<Setting>,
}

impl Request for ReadSettings {
    type Response = ReadSettingsResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `delete_setting` request (doc/interfaces.md#delete_setting, schemas/database_handler/delete_setting.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteSetting {
    pub name: String,
}

/// `delete_setting_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteSettingResponse {
    pub name: String,
}

impl Request for DeleteSetting {
    type Response = DeleteSettingResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(5);
}

/// `reset_settings` request (doc/interfaces.md#reset_settings, schemas/database_handler/reset_settings.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResetSettings {
    // TODO: fields
}

/// `reset_settings_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResetSettingsResponse {
    // TODO: fields
}

impl Request for ResetSettings {
    type Response = ResetSettingsResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(300);
}

/// `change_image_dir` request (doc/interfaces.md#change_image_dir, schemas/database_handler/change_image_dir.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeImageDir {
    // TODO: fields
}

/// `change_image_dir_response`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeImageDirResponse {
    // TODO: fields
}

impl Request for ChangeImageDir {
    type Response = ChangeImageDirResponse;
    const STREAM: &'static str = STREAM;
    const TIMEOUT: Duration = Duration::from_secs(300);
}

/// `data_changed` event (doc/interfaces.md#data_changed, schemas/database_handler/data_changed.schema.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataChanged {
    pub table: String,
    pub key: Option<serde_json::Value>,
    pub action: String,
}
