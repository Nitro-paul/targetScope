// doc/programs/statistics.md#calculation-flow-compute_statistics
// doc/interfaces.md#statisticsgroup

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use serde_json::{json, Value};
use targetscope_common::messages::statistics::GroupBy;
use targetscope_common::model::{Device, Session};

pub struct Group<'a> {
    pub target_type_id: i64,
    pub group_key: Option<Value>,
    pub sessions: Vec<&'a Session>,
}

pub fn key(s: &Session, by: GroupBy, devices: &HashMap<i64, Device>) -> Option<Value> {
    match by {
        GroupBy::None => None,
        GroupBy::Shooter => Some(json!({ "shooter_id": s.shooter_id })),
        GroupBy::Device => Some(json!({ "device_id": s.device_id })),
        GroupBy::DeviceCategory => {
            Some(json!({ "category": devices.get(&s.device_id).and_then(|d| d.category.clone()) }))
        }
        GroupBy::Distance => Some(json!({ "distance_m": s.distance_m })),
        GroupBy::SingleHanded => Some(json!({ "single_handed": s.single_handed })),
        GroupBy::Supported => Some(json!({ "supported": s.supported })),
    }
}

// doc/programs/statistics.md#calculation-flow-compute_statistics
pub fn compare_keys(a: &Option<Value>, b: &Option<Value>) -> Ordering {
    let inner = |v: &Option<Value>| v.as_ref().and_then(|o| o.as_object()).and_then(|m| m.values().next().cloned());
    match (inner(a), inner(b)) {
        (None, None) | (Some(Value::Null), Some(Value::Null)) => Ordering::Equal,
        (None | Some(Value::Null), _) => Ordering::Greater,
        (_, None | Some(Value::Null)) => Ordering::Less,
        (Some(Value::Number(x)), Some(Value::Number(y))) => x.as_f64().unwrap_or(0.0).total_cmp(&y.as_f64().unwrap_or(0.0)),
        (Some(Value::String(x)), Some(Value::String(y))) => x.to_lowercase().cmp(&y.to_lowercase()).then(x.cmp(&y)),
        (Some(Value::Bool(x)), Some(Value::Bool(y))) => x.cmp(&y),
        (Some(x), Some(y)) => x.to_string().cmp(&y.to_string()),
    }
}

pub fn group_sessions<'a>(
    sessions: &'a [Session],
    by: GroupBy,
    devices: &HashMap<i64, Device>,
    target_names: &HashMap<i64, String>,
) -> Vec<Group<'a>> {
    let mut map: BTreeMap<(i64, String), Group<'a>> = BTreeMap::new();
    for s in sessions {
        let k = key(s, by, devices);
        let entry = map.entry((s.target_type_id, k.as_ref().map(Value::to_string).unwrap_or_default())).or_insert_with(|| Group {
            target_type_id: s.target_type_id,
            group_key: k.clone(),
            sessions: Vec::new(),
        });
        entry.sessions.push(s);
    }
    let mut groups: Vec<Group<'a>> = map.into_values().collect();
    let name = |id: i64| target_names.get(&id).map(|n| n.to_lowercase()).unwrap_or_default();
    groups.sort_by(|a, b| {
        name(a.target_type_id)
            .cmp(&name(b.target_type_id))
            .then(a.target_type_id.cmp(&b.target_type_id))
            .then(compare_keys(&a.group_key, &b.group_key))
    });
    groups
}
