//! The known settings (doc/database.md#known-settings): name, display name, default value,
//! default active, optional, value rule.

pub struct KnownSetting {
    pub name: &'static str,
    pub display_name: &'static str,
    /// May contain "<data_dir>", replaced at startup.
    pub default_value: &'static str,
    pub default_active: bool,
    pub optional: bool,
    pub value_rule: &'static str,
}

/// TODO: all 20 known settings from doc/database.md#known-settings.
pub const KNOWN_SETTINGS: &[KnownSetting] = &[];

pub fn find(name: &str) -> Option<&'static KnownSetting> {
    KNOWN_SETTINGS.iter().find(|s| s.name == name)
}
