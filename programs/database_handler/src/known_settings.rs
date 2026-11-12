// doc/database.md#known-settings

use std::net::SocketAddrV4;
use std::path::Path;

use targetscope_common::model::Setting;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Folder,
    StagingFolder,
    Integer(i64, i64),
    OneOf(&'static [&'static str]),
    Text,
    RtspUrl,
    JsonObject,
    BindAddress,
    Argon2Hash,
}

pub struct KnownSetting {
    pub name: &'static str,
    pub display_name: &'static str,
    pub default_value: &'static str,
    pub default_active: bool,
    pub optional: bool,
    pub rule: Rule,
    pub value_rule: &'static str,
}

const fn s(
    name: &'static str,
    display_name: &'static str,
    default_value: &'static str,
    optional: bool,
    default_active: bool,
    rule: Rule,
    value_rule: &'static str,
) -> KnownSetting {
    KnownSetting { name, display_name, default_value, default_active, optional, rule, value_rule }
}

pub const KNOWN_SETTINGS: &[KnownSetting] = &[
    s("image_dir", "Image folder", "<data_dir>/images", false, true, Rule::Folder, "folder"),
    s("staging_dir", "Staging folder", "<data_dir>/staging", false, true, Rule::StagingFolder, "folder, same file system as image_dir"),
    s("export_dir", "Export folder", "~/TargetScope/exports", true, false, Rule::Folder, "folder"),
    s("processed_requests_retention_days", "Keep processed requests (days)", "30", false, true, Rule::Integer(1, 3650), "integer 1..=3650"),
    s("staging_max_age_hours", "Delete staged files after (hours)", "24", false, true, Rule::Integer(1, 8760), "integer 1..=8760"),
    s("camera_source", "Camera source", "v4l2", false, true, Rule::OneOf(&["v4l2", "rtsp", "none"]), "one of v4l2, rtsp, none"),
    s("camera_device", "USB camera", "", false, true, Rule::Text, "text"),
    s("camera_rtsp_url", "Network camera URL", "", true, false, Rule::RtspUrl, "URL starting with rtsp://"),
    s("camera_width_px", "Frame width (px)", "1920", false, true, Rule::Integer(160, 7680), "integer 160..=7680"),
    s("camera_height_px", "Frame height (px)", "1080", false, true, Rule::Integer(120, 4320), "integer 120..=4320"),
    s("camera_fps", "Frame rate (fps)", "30", false, true, Rule::Integer(1, 120), "integer 1..=120"),
    s("camera_controls", "Camera controls", "{}", false, true, Rule::JsonObject, "JSON object"),
    s("lens_calibrations", "Lens calibrations", "{}", false, true, Rule::JsonObject, "JSON object"),
    s("live_long_side_px", "Live image size (px)", "1200", false, true, Rule::Integer(400, 4000), "integer 400..=4000"),
    s("capture_long_side_px", "Stored image size (px)", "2400", false, true, Rule::Integer(400, 8000), "integer 400..=8000"),
    s("detection_confirm_frames", "Frames to confirm a hole", "5", false, true, Rule::Integer(1, 60), "integer 1..=60"),
    s("live_feed_fps", "Live feed frame rate (fps)", "15", false, true, Rule::Integer(1, 60), "integer 1..=60"),
    s("live_feed_quality", "Live feed JPEG quality", "80", false, true, Rule::Integer(1, 100), "integer 1..=100"),
    s("web_bind_address", "Web UI address", "0.0.0.0:8080", false, true, Rule::BindAddress, "<IPv4 address>:<port>"),
    s("admin_password_hash", "Admin password", "", true, false, Rule::Argon2Hash, "Argon2id hash (PHC string)"),
];

// doc/interfaces.md#reset_settings
pub const NOT_RESET_BY_DEFAULT: &[&str] = &["admin_password_hash", "lens_calibrations"];

pub fn find(name: &str) -> Option<&'static KnownSetting> {
    KNOWN_SETTINGS.iter().find(|s| s.name == name)
}

impl KnownSetting {
    pub fn default_value(&self, data_dir: &Path) -> String {
        self.default_value.replace("<data_dir>", &data_dir.display().to_string())
    }
}

// doc/interfaces.md#setting
pub fn complete(mut s: Setting, data_dir: &Path) -> Setting {
    match find(&s.name) {
        Some(k) => {
            s.optional = k.optional;
            s.default_value = k.default_value(data_dir);
            s.default_active = k.default_active;
            s.value_rule = k.value_rule.to_string();
        }
        None => {
            s.optional = false;
            s.default_value = String::new();
            s.default_active = true;
            s.value_rule = "text".to_string();
        }
    }
    s
}

// doc/database.md#known-settings
pub fn check_value(rule: Rule, value: &str) -> Result<(), String> {
    let ok = match rule {
        Rule::Folder | Rule::StagingFolder => !value.trim().is_empty(),
        Rule::Integer(min, max) => value.parse::<i64>().is_ok_and(|v| (min..=max).contains(&v)),
        Rule::OneOf(values) => values.contains(&value),
        Rule::Text => true,
        Rule::RtspUrl => value.strip_prefix("rtsp://").is_some_and(|rest| !rest.is_empty() && !rest.contains(char::is_whitespace)),
        Rule::JsonObject => serde_json::from_str::<serde_json::Value>(value).is_ok_and(|v| v.is_object()),
        Rule::BindAddress => value.parse::<SocketAddrV4>().is_ok(),
        Rule::Argon2Hash => is_argon2id_phc(value),
    };
    if ok {
        Ok(())
    } else {
        Err(format!("value doesn't match the rule: {}", describe(rule)))
    }
}

fn describe(rule: Rule) -> String {
    KNOWN_SETTINGS.iter().find(|k| k.rule == rule).map(|k| k.value_rule.to_string()).unwrap_or_else(|| "text".into())
}

// doc/database.md#known-settings
fn is_argon2id_phc(v: &str) -> bool {
    let parts: Vec<&str> = v.split('$').collect();
    parts.len() == 6
        && parts[0].is_empty()
        && parts[1] == "argon2id"
        && parts[2].starts_with("v=")
        && parts[3].contains("m=")
        && !parts[4].is_empty()
        && !parts[5].is_empty()
}
