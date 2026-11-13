// doc/testing/shared_libraries.md#envelopes-unit

use serde_json::{json, Value};
use targetscope_common::messages::analyzer::AnalyzerRequest;
use targetscope_common::messages::image_processor::{ImageProcessorEvent, ShotDetected};
use targetscope_common::messages::{ErrorCode, EventEnvelope, Ping, RequestEnvelope, ServiceError, Status};
use uuid::Uuid;

fn ping_envelope(reply_to: Option<&str>) -> RequestEnvelope {
    RequestEnvelope {
        r#type: "ping".into(),
        version: 1,
        request_id: Uuid::new_v4(),
        reply_to: reply_to.map(str::to_string),
        payload: serde_json::to_value(Ping {}).unwrap(),
    }
}

// LIB-ENV-01
#[test]
fn lib_env_01_request_envelope() {
    let v = serde_json::to_value(ping_envelope(Some("a:responses"))).unwrap();
    let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
    for k in ["type", "version", "request_id", "reply_to", "payload"] {
        assert!(keys.contains(&k), "{k}");
    }
    assert_eq!(keys.len(), 5);
    let id = Uuid::parse_str(v["request_id"].as_str().unwrap()).unwrap();
    assert_eq!(id.get_version_num(), 4);
}

// LIB-ENV-02
#[test]
fn lib_env_02_request_without_reply_to() {
    let v = serde_json::to_value(ping_envelope(None)).unwrap();
    assert!(v.get("reply_to").is_none_or(Value::is_null));
}

// LIB-ENV-03
#[test]
fn lib_env_03_response_type_name() {
    let req = RequestEnvelope { r#type: "create_session".into(), ..ping_envelope(Some("x:responses")) };
    let resp = req.ok_response(json!({}));
    assert_eq!(resp.r#type, "create_session_response");
    assert_eq!(resp.request_id, req.request_id);
    assert_eq!(resp.version, req.version);
}

// LIB-ENV-04
#[test]
fn lib_env_04_ok_response() {
    let resp = ping_envelope(Some("x:responses")).ok_response(json!({"a": 1}));
    let v = serde_json::to_value(&resp).unwrap();
    assert_eq!(v["status"], "ok");
    assert!(v["error"].is_null());
    assert_eq!(v["payload"], json!({"a": 1}));
}

// LIB-ENV-05
#[test]
fn lib_env_05_error_response() {
    let err = ServiceError::setting_not_set("export_dir");
    let resp = ping_envelope(Some("x:responses")).error_response(&err);
    let v = serde_json::to_value(&resp).unwrap();
    assert_eq!(resp.status, Status::Error);
    assert!(v["payload"].is_null());
    assert_eq!(v["error"]["code"], "setting_not_set");
    assert!(!v["error"]["message"].as_str().unwrap().is_empty());
    assert_eq!(v["error"]["details"], json!({"setting": "export_dir"}));
}

// LIB-ENV-06
#[test]
fn lib_env_06_event_envelope() {
    let ev = ImageProcessorEvent::ShotDetected(ShotDetected {
        hole_id: 1,
        live_paper: 1,
        angle_turns: 0.25,
        radius_mm: 10.0,
        confidence: 0.9,
        frame_seq: 7,
    });
    let env = EventEnvelope::new("image_processor", 1_794_393_600_123, &ev).unwrap();
    let v = serde_json::to_value(&env).unwrap();
    assert_eq!(v["type"], "shot_detected");
    assert_eq!(v["version"], 1);
    assert_eq!(v["source"], "image_processor");
    assert_eq!(v["timestamp_ms"], 1_794_393_600_123i64);
    assert_eq!(v["payload"]["hole_id"], 1);
    assert_eq!(env.event_id.get_version_num(), 4);
    assert_eq!(env.decode::<ImageProcessorEvent>().unwrap(), ev);
}

// LIB-ENV-07
#[test]
fn lib_env_07_unknown_request_type() {
    let env = RequestEnvelope { r#type: "nonsense".into(), ..ping_envelope(None) };
    assert_eq!(env.decode::<AnalyzerRequest>().unwrap_err().code, ErrorCode::InvalidMessage);
}

// LIB-ENV-08
#[test]
fn lib_env_08_unknown_version() {
    let env = RequestEnvelope { version: 99, ..ping_envelope(None) };
    assert_eq!(env.decode::<AnalyzerRequest>().unwrap_err().code, ErrorCode::InvalidMessage);
}
