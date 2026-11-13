// doc/interfaces.md#database_handler

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{request, Ping};
use crate::model::{
    Device, Filter, ImportedSession, NewSession, PaperImage, Session, SessionFilter, Setting, Shooter, TargetType,
};

pub const PROGRAM: &str = "database_handler";
pub const STREAM: &str = "database_handler:requests";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DatabaseHandlerRequest {
    Ping(Ping),
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

// doc/interfaces.md#events-database_handlerevents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DatabaseHandlerEvent {
    DataChanged(DataChanged),
}

// doc/interfaces.md#create_session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSession(pub NewSession);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSessionResponse {
    pub id: i64,
    pub uuid: Uuid,
    pub images: Vec<PaperImage>,
}
request!(CreateSession, CreateSessionResponse, "create_session", 5);

// doc/interfaces.md#read_sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionOrderField {
    SavedAt,
    Score,
    ShotCount,
    Id,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionOrder {
    pub field: SessionOrderField,
    pub direction: crate::model::Direction,
}

// doc/interfaces.md#read_sessions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadSessions {
    #[serde(default)]
    pub filter: SessionFilter,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_by: Option<SessionOrder>,
    #[serde(default, deserialize_with = "crate::check::opt_count", skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(default, deserialize_with = "crate::check::opt_count", skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default = "default_true")]
    pub include_shots: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadSessionsResponse {
    pub sessions: Vec<Session>,
    pub total: i64,
}
request!(ReadSessions, ReadSessionsResponse, "read_sessions", 5);

// doc/interfaces.md#delete_session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteSession {
    pub id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdResponse {
    pub id: i64,
}
request!(DeleteSession, IdResponse, "delete_session", 5);

// doc/interfaces.md#save_shooter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveShooter(pub Shooter);
request!(SaveShooter, IdResponse, "save_shooter", 5);

// doc/interfaces.md#read_shooters
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadShooters {
    #[serde(default)]
    pub filter: Filter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadShootersResponse {
    pub shooters: Vec<Shooter>,
}
request!(ReadShooters, ReadShootersResponse, "read_shooters", 5);

// doc/interfaces.md#delete_shooter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteShooter {
    pub id: i64,
}
request!(DeleteShooter, IdResponse, "delete_shooter", 5);

// doc/interfaces.md#save_device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveDevice(pub Device);
request!(SaveDevice, IdResponse, "save_device", 5);

// doc/interfaces.md#read_devices
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadDevices {
    #[serde(default)]
    pub filter: Filter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadDevicesResponse {
    pub devices: Vec<Device>,
}
request!(ReadDevices, ReadDevicesResponse, "read_devices", 5);

// doc/interfaces.md#delete_device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteDevice {
    pub id: i64,
}
request!(DeleteDevice, IdResponse, "delete_device", 5);

// doc/interfaces.md#set_device_image
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetDeviceImage {
    pub device_id: i64,
    pub staged_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetDeviceImageResponse {
    pub image_path: String,
}
request!(SetDeviceImage, SetDeviceImageResponse, "set_device_image", 5);

// doc/interfaces.md#delete_device_image
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteDeviceImage {
    pub device_id: i64,
}
request!(DeleteDeviceImage, DeleteDeviceImage, "delete_device_image", 5);

// doc/interfaces.md#save_target_type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveTargetType(pub TargetType);
request!(SaveTargetType, IdResponse, "save_target_type", 5);

// doc/interfaces.md#read_target_types
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadTargetTypes {
    #[serde(default)]
    pub filter: Filter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadTargetTypesResponse {
    pub target_types: Vec<TargetType>,
}
request!(ReadTargetTypes, ReadTargetTypesResponse, "read_target_types", 5);

// doc/interfaces.md#delete_target_type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteTargetType {
    pub id: i64,
}
request!(DeleteTargetType, IdResponse, "delete_target_type", 5);

// doc/interfaces.md#import
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportResponse {
    pub id: i64,
    pub created: bool,
}

// doc/interfaces.md#import_shooter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportShooter(pub Shooter);
request!(ImportShooter, ImportResponse, "import_shooter", 5);

// doc/interfaces.md#import_target_type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportTargetType(pub TargetType);
request!(ImportTargetType, ImportResponse, "import_target_type", 5);

// doc/interfaces.md#import_device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "serde_json::Map<String, Value>")]
pub struct ImportDevice {
    #[serde(flatten)]
    pub device: Device,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staged_file: Option<String>,
}

// doc/shared_libraries.md#value-checks
impl TryFrom<serde_json::Map<String, Value>> for ImportDevice {
    type Error = String;

    fn try_from(mut m: serde_json::Map<String, Value>) -> Result<Self, String> {
        let staged_file = match m.remove("staged_file") {
            Some(v) => serde_json::from_value(v).map_err(|e| e.to_string())?,
            None => None,
        };
        let device = serde_json::from_value(Value::Object(m)).map_err(|e| e.to_string())?;
        Ok(ImportDevice { device, staged_file })
    }
}
request!(ImportDevice, ImportResponse, "import_device", 5);

// doc/interfaces.md#import_session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportSession(pub ImportedSession);
request!(ImportSession, ImportResponse, "import_session", 5);

// doc/interfaces.md#save_setting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveSetting(pub Setting);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NameResponse {
    pub name: String,
}
request!(SaveSetting, NameResponse, "save_setting", 5);

// doc/interfaces.md#read_settings
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadSettings {
    #[serde(default)]
    pub filter: Filter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadSettingsResponse {
    pub settings: Vec<Setting>,
}
request!(ReadSettings, ReadSettingsResponse, "read_settings", 5);

// doc/interfaces.md#delete_setting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteSetting {
    pub name: String,
}
request!(DeleteSetting, NameResponse, "delete_setting", 5);

// doc/interfaces.md#reset_settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResetSettings {
    pub names: Option<Vec<String>>,
    #[serde(default)]
    pub move_images: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovedImages {
    pub moved: i64,
    pub failed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResetSettingsResponse {
    pub names: Vec<String>,
    pub images: Option<MovedImages>,
}
request!(ResetSettings, ResetSettingsResponse, "reset_settings", 300);

// doc/interfaces.md#change_image_dir
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeImageDir {
    pub path: String,
    #[serde(default)]
    pub move_images: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeImageDirResponse {
    pub path: String,
    pub images: Option<MovedImages>,
}
request!(ChangeImageDir, ChangeImageDirResponse, "change_image_dir", 300);

// doc/interfaces.md#data_changed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Table {
    Sessions,
    Shooters,
    Devices,
    TargetTypes,
    Settings,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeAction {
    Created,
    Updated,
    Deleted,
    Reload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataChanged {
    pub table: Table,
    pub key: serde_json::Value,
    pub action: ChangeAction,
}

fn default_true() -> bool {
    true
}
