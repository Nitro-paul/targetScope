//! Step 3: group by target_type_id and the group_by key. Sessions without a shooter form their
//! own group ({"shooter_id": null}). Groups sorted by target type name, then group key.

use serde::Deserialize;
use targetscope_common::model::{Device, Session};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupBy {
    None,
    Shooter,
    Device,
    DeviceCategory,
    Distance,
    SingleHanded,
    Supported,
}

pub struct Group<'a> {
    pub target_type_id: i64,
    pub group_key: Option<serde_json::Value>,
    pub sessions: Vec<&'a Session>,
}

pub fn group_sessions<'a>(
    sessions: &'a [Session],
    devices: &[Device],
    by: GroupBy,
) -> Vec<Group<'a>> {
    let _ = (sessions, devices, by);
    todo!()
}
