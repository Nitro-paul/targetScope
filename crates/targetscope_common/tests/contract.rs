// doc/testing/shared_libraries.md#contract-schemas-and-examples-contract
// doc/shared_libraries.md#contract-tests

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};

use targetscope_common::export::ExportFile;
use targetscope_common::messages::{
    analyzer, camera_reader, database_handler as db, exporter, image_processor as ip, importer, launcher,
    session_manager as sm, statistics, ErrorBody, ErrorCode, EventEnvelope, Ping, PingResponse, Request,
    RequestEnvelope, ResponseEnvelope, ServiceError, Status, VERSION,
};
use targetscope_common::model;

const BASE: &str = "https://targetscope.invalid/schemas/";

fn schemas_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas")
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(json_files(&path));
        } else if path.extension().is_some_and(|e| e == "json") {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn rel(path: &Path, base: &Path) -> String {
    path.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/")
}

fn schema_names() -> BTreeSet<String> {
    let dir = schemas_dir();
    json_files(&dir)
        .iter()
        .map(|p| rel(p, &dir))
        .filter_map(|r| r.strip_suffix(".schema.json").map(str::to_string))
        .collect()
}


struct SchemaFiles;

impl jsonschema::Retrieve for SchemaFiles {
    fn retrieve(&self, uri: &jsonschema::Uri<String>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let path = uri.as_str().strip_prefix(BASE).ok_or_else(|| format!("{uri} is outside the schemas"))?;
        Ok(read_json(&schemas_dir().join(path)))
    }
}

fn validators() -> &'static BTreeMap<String, jsonschema::Validator> {
    static V: OnceLock<BTreeMap<String, jsonschema::Validator>> = OnceLock::new();
    V.get_or_init(|| {
        schema_names()
            .into_iter()
            .map(|name| {
                let schema = read_json(&schemas_dir().join(format!("{name}.schema.json")));
                let validator = jsonschema::options()
                    .with_draft(jsonschema::Draft::Draft202012)
                    .with_retriever(SchemaFiles)
                    .build(&schema)
                    .unwrap_or_else(|e| panic!("{name}: {e}"));
                (name, validator)
            })
            .collect()
    })
}

fn errors(schema: &str, instance: &Value) -> Vec<String> {
    let v = validators().get(schema).unwrap_or_else(|| panic!("no schema {schema}"));
    v.iter_errors(instance).map(|e| format!("{}: {}", e.instance_path(), e)).collect()
}

fn schema(name: &str) -> Value {
    read_json(&schemas_dir().join(format!("{name}.schema.json")))
}


#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Shared,
    Request,
    Response,
    Event,
}

type Codec = fn(&Value) -> Result<Value, String>;

#[derive(Clone, Copy)]
struct Entry {
    kind: Kind,
    round_trip: Codec,
    produce: Codec,
    receive: Codec,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn bare<T: Serialize + DeserializeOwned>(v: &Value) -> Result<Value, String> {
    let t: T = serde_json::from_value(v.clone()).map_err(err)?;
    serde_json::to_value(&t).map_err(err)
}

fn shared<T: Serialize + DeserializeOwned>() -> Entry {
    Entry { kind: Kind::Shared, round_trip: bare::<T>, produce: bare::<T>, receive: bare::<T> }
}

fn request_round_trip<T: Serialize + DeserializeOwned>(m: &Value) -> Result<Value, String> {
    let mut env: RequestEnvelope = serde_json::from_value(m.clone()).map_err(err)?;
    env.payload = bare::<T>(&env.payload)?;
    serde_json::to_value(&env).map_err(err)
}

fn request_produce<T: Request + DeserializeOwned>(m: &Value) -> Result<Value, String> {
    let env: RequestEnvelope = serde_json::from_value(m.clone()).map_err(err)?;
    let t: T = serde_json::from_value(env.payload.clone()).map_err(err)?;
    serde_json::to_value(RequestEnvelope::new(env.request_id, env.reply_to.clone(), &t)).map_err(err)
}

fn request_receive<E: Serialize + DeserializeOwned>(m: &Value) -> Result<Value, String> {
    let env: RequestEnvelope = serde_json::from_value(m.clone()).map_err(err)?;
    let decoded: E = env.decode().map_err(err)?;
    serde_json::to_value(&decoded).map_err(err)
}

fn request<T: Request + DeserializeOwned, E: Serialize + DeserializeOwned>() -> Entry {
    Entry {
        kind: Kind::Request,
        round_trip: request_round_trip::<T>,
        produce: request_produce::<T>,
        receive: request_receive::<E>,
    }
}

fn response_round_trip<R: Serialize + DeserializeOwned>(m: &Value) -> Result<Value, String> {
    let mut env: ResponseEnvelope = serde_json::from_value(m.clone()).map_err(err)?;
    if let Some(p) = &env.payload {
        env.payload = Some(bare::<R>(p)?);
    }
    serde_json::to_value(&env).map_err(err)
}

fn response_produce<R: Serialize + DeserializeOwned>(m: &Value) -> Result<Value, String> {
    let env: ResponseEnvelope = serde_json::from_value(m.clone()).map_err(err)?;
    let request_type = env.r#type.strip_suffix("_response").ok_or("type without _response")?;
    let req = RequestEnvelope {
        r#type: request_type.to_string(),
        version: VERSION,
        request_id: env.request_id,
        reply_to: None,
        payload: json!({}),
    };
    let produced = match (env.status, env.payload, env.error) {
        (Status::Ok, Some(p), _) => {
            let r: R = serde_json::from_value(p).map_err(err)?;
            req.ok_response(serde_json::to_value(&r).map_err(err)?)
        }
        (Status::Error, _, Some(e)) => req.error_response(&ServiceError::from(e)),
        _ => return Err("inconsistent response".into()),
    };
    serde_json::to_value(produced).map_err(err)
}

// doc/shared_libraries.md#redis-bus
fn response_receive<R: Serialize + DeserializeOwned>(m: &Value) -> Result<Value, String> {
    let env: ResponseEnvelope = serde_json::from_value(m.clone()).map_err(err)?;
    if env.version != VERSION {
        return Err(format!("version {}", env.version));
    }
    match env.status {
        Status::Ok => bare::<R>(&env.payload.ok_or("ok response without payload")?),
        Status::Error => Ok(serde_json::to_value(env.error.ok_or("error response without error")?).map_err(err)?),
    }
}

fn response<R: Serialize + DeserializeOwned>() -> Entry {
    Entry {
        kind: Kind::Response,
        round_trip: response_round_trip::<R>,
        produce: response_produce::<R>,
        receive: response_receive::<R>,
    }
}

fn event_round_trip<E: Serialize + DeserializeOwned>(m: &Value) -> Result<Value, String> {
    let mut env: EventEnvelope = serde_json::from_value(m.clone()).map_err(err)?;
    let event: E = env.decode().map_err(err)?;
    let tagged = serde_json::to_value(&event).map_err(err)?;
    env.payload = tagged.get("payload").cloned().ok_or("event without payload")?;
    serde_json::to_value(&env).map_err(err)
}

fn event_produce<E: Serialize + DeserializeOwned>(m: &Value) -> Result<Value, String> {
    let env: EventEnvelope = serde_json::from_value(m.clone()).map_err(err)?;
    let event: E = env.decode().map_err(err)?;
    let produced = EventEnvelope::new(&env.source, env.timestamp_ms, &event).map_err(err)?;
    serde_json::to_value(produced).map_err(err)
}

fn event_receive<E: Serialize + DeserializeOwned>(m: &Value) -> Result<Value, String> {
    let env: EventEnvelope = serde_json::from_value(m.clone()).map_err(err)?;
    let event: E = env.decode().map_err(err)?;
    serde_json::to_value(&event).map_err(err)
}

fn event<E: Serialize + DeserializeOwned>() -> Entry {
    Entry { kind: Kind::Event, round_trip: event_round_trip::<E>, produce: event_produce::<E>, receive: event_receive::<E> }
}

// doc/shared_libraries.md#contract-tests
fn registry() -> &'static BTreeMap<String, Entry> {
    static R: OnceLock<BTreeMap<String, Entry>> = OnceLock::new();
    R.get_or_init(|| {
        let mut r = BTreeMap::new();
        macro_rules! shared {
            ($($name:literal => $t:ty),* $(,)?) => { $( r.insert(format!("common/{}", $name), shared::<$t>()); )* };
        }
        macro_rules! requests {
            ($program:path, $enum:ty: $($t:ty),* $(,)?) => { $(
                {
                    use $program as p;
                    assert_eq!(<$t as Request>::STREAM, format!("{}:requests", p::PROGRAM));
                    r.insert(format!("{}/{}", p::PROGRAM, <$t as Request>::TYPE), request::<$t, $enum>());
                    r.insert(
                        format!("{}/{}_response", p::PROGRAM, <$t as Request>::TYPE),
                        response::<<$t as Request>::Response>(),
                    );
                }
            )* };
        }
        macro_rules! events {
            ($program:path, $enum:ty: $($name:literal),* $(,)?) => { $(
                {
                    use $program as p;
                    r.insert(format!("{}/{}", p::PROGRAM, $name), event::<$enum>());
                }
            )* };
        }

        shared! {
            "request_envelope" => RequestEnvelope, "response_envelope" => ResponseEnvelope,
            "event_envelope" => EventEnvelope, "error" => ErrorBody, "error_code" => ErrorCode,
            "shot" => model::Shot, "session_shot" => model::SessionShot, "paper_image" => model::PaperImage,
            "staged_paper_image" => model::StagedPaperImage, "session" => model::Session,
            "new_session" => model::NewSession, "shooter" => model::Shooter, "device" => model::Device,
            "ring" => model::Ring, "target_type" => model::TargetType, "setting" => model::Setting,
            "condition" => model::Condition, "order_by" => model::OrderBy, "filter" => model::Filter,
            "int_range" => model::IntRange, "time_range" => model::TimeRange,
            "session_filter" => model::SessionFilter, "pixel_point" => model::PixelPoint,
            "corners" => model::Corners, "offset_mm" => model::OffsetMm,
            "analysis_result" => model::AnalysisResult, "session_setup" => model::SessionSetup,
            "current_shot" => model::CurrentShot, "calibration_status" => model::CalibrationStatus,
            "current_session" => model::CurrentSession, "menu_entry" => model::MenuEntry,
            "camera_control" => model::CameraControl, "statistics_values" => model::StatisticsValues,
            "trend_entry" => model::TrendEntry, "statistics_group" => model::StatisticsGroup,
        }
        r.insert("export/v1".into(), shared::<ExportFile>());
        // doc/interfaces.md#ping
        r.insert(
            "common/ping".into(),
            Entry {
                kind: Kind::Request,
                round_trip: request_round_trip::<Ping>,
                produce: request_round_trip::<Ping>,
                receive: request_receive::<analyzer::AnalyzerRequest>,
            },
        );
        r.insert("common/ping_response".into(), response::<PingResponse>());

        requests!(analyzer, analyzer::AnalyzerRequest: analyzer::AnalyzeShots);
        requests!(camera_reader, camera_reader::CameraReaderRequest:
            camera_reader::CameraStatus, camera_reader::CameraListDevices, camera_reader::CameraListControls,
            camera_reader::CameraSetControl);
        events!(camera_reader, camera_reader::CameraReaderEvent: "camera_state_changed");
        requests!(db, db::DatabaseHandlerRequest:
            db::CreateSession, db::ReadSessions, db::DeleteSession, db::SaveShooter, db::ReadShooters,
            db::DeleteShooter, db::SaveDevice, db::ReadDevices, db::DeleteDevice, db::SetDeviceImage,
            db::DeleteDeviceImage, db::SaveTargetType, db::ReadTargetTypes, db::DeleteTargetType,
            db::ImportShooter, db::ImportTargetType, db::ImportDevice, db::ImportSession, db::SaveSetting,
            db::ReadSettings, db::DeleteSetting, db::ResetSettings, db::ChangeImageDir);
        events!(db, db::DatabaseHandlerEvent: "data_changed");
        requests!(exporter, exporter::ExporterRequest:
            exporter::ExportSessions, exporter::ListExports, exporter::DeleteExport);
        requests!(ip, ip::ImageProcessorRequest:
            ip::LiveStart, ip::LiveCalibrate, ip::LiveSetCounting, ip::LiveUncountHole, ip::LiveNewPaper,
            ip::LiveStop, ip::LiveCalibrationStatus, ip::LensCalibrationStart, ip::LensCalibrationCapture,
            ip::LensCalibrationFinish, ip::LensCalibrationCancel, ip::LensCalibrationDelete,
            ip::LensCalibrationStatus, ip::CaptureTargetImage, ip::CaptureRawImage, ip::FindPaper,
            ip::PrepareTargetPhoto, ip::DetectShotsInPhoto);
        events!(ip, ip::ImageProcessorEvent: "calibration_changed", "paper_replaced", "shot_detected");
        requests!(importer, importer::ImporterRequest: importer::ImportFile);
        requests!(launcher, launcher::LauncherRequest: launcher::LauncherStatus, launcher::RestartProgram);
        events!(launcher, launcher::LauncherEvent: "program_state_changed");
        requests!(sm, sm::SessionManagerRequest:
            sm::SessionGet, sm::SessionSetupRequest, sm::SessionUpdateSetup, sm::SessionCalibrate,
            sm::SessionStart, sm::SessionUploadPhoto, sm::SessionPause, sm::SessionResume, sm::SessionAddShot,
            sm::SessionUpdateShot, sm::SessionDeleteShot, sm::SessionNewPaper, sm::SessionSetNotes,
            sm::SessionStop, sm::SessionSave, sm::SessionCancel);
        events!(sm, sm::SessionManagerEvent: "session_changed", "session_saved");
        requests!(statistics, statistics::StatisticsRequest: statistics::ComputeStatistics);
        r
    })
}

// doc/shared_libraries.md#contract-tests
fn variants<E: DeserializeOwned>() -> Vec<String> {
    let e = serde_json::from_value::<E>(json!({ "type": "\u{1}", "payload": {} })).err().expect("unknown variant");
    let msg = e.to_string();
    let list = msg.split("expected one of ").nth(1).or_else(|| msg.split("expected ").nth(1)).unwrap_or("");
    let list = list.split(" at line ").next().unwrap_or("");
    list.split([',', ' '])
        .map(|s| s.trim().trim_matches('`'))
        .filter(|s| !s.is_empty() && *s != "or")
        .map(str::to_string)
        .collect()
}


fn examples_dir() -> PathBuf {
    schemas_dir().join("examples")
}

fn examples() -> BTreeMap<String, Value> {
    let dir = examples_dir();
    json_files(&dir)
        .into_iter()
        .filter_map(|p| {
            let r = rel(&p, &dir);
            if r.starts_with("invalid/") || r.starts_with("target_types/") {
                return None;
            }
            Some((r.strip_suffix(".json").unwrap().to_string(), read_json(&p)))
        })
        .collect()
}

struct Invalid {
    file: String,
    schema: String,
    expected_error: String,
    message: Value,
}

fn invalid_examples() -> Vec<Invalid> {
    let dir = examples_dir().join("invalid");
    json_files(&dir)
        .into_iter()
        .map(|p| {
            let r = rel(&p, &dir);
            let v = read_json(&p);
            let (stem, _) = r.rsplit_once('.').unwrap(); // drop ".json"
            let (schema, _case) = stem.rsplit_once('.').unwrap_or_else(|| panic!("{r}: name <schema>.<case>.json"));
            Invalid {
                file: r.clone(),
                schema: schema.to_string(),
                expected_error: v["expected_error"].as_str().unwrap_or_else(|| panic!("{r}: expected_error")).into(),
                message: v["message"].clone(),
            }
        })
        .collect()
}

fn json_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(a, b)| json_eq(a, b)),
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| json_eq(v, w)))
        }
        _ => a == b,
    }
}

fn report(failures: Vec<String>) {
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
}


fn payload_schema(name: &str, kind: Kind) -> Value {
    let s = schema(name);
    let mut p = if kind == Kind::Shared { s } else { s["$defs"]["payload"].clone() };
    if p.get("$ref").and_then(Value::as_str) == Some("#/$defs/program_status") {
        p = schema(name)["$defs"]["program_status"].clone();
    }
    if let Some(r) = p.get("$ref").and_then(Value::as_str) {
        let file = r.trim_start_matches("../").trim_start_matches("common/").trim_end_matches(".schema.json");
        p = schema(&format!("common/{file}"));
    }
    p
}

fn allows_null(s: &Value) -> bool {
    let t = &s["type"];
    t == "null"
        || t.as_array().is_some_and(|a| a.iter().any(|x| x == "null"))
        || ["oneOf", "anyOf"].iter().any(|k| s[*k].as_array().is_some_and(|a| a.iter().any(allows_null)))
}

fn enum_values(s: &Value) -> Vec<Value> {
    let mut out: Vec<Value> = s["enum"].as_array().cloned().unwrap_or_default();
    for k in ["oneOf", "anyOf"] {
        for alt in s[k].as_array().into_iter().flatten() {
            out.extend(enum_values(alt));
        }
    }
    if s["type"] == "boolean" {
        out.extend([json!(true), json!(false)]);
    }
    out
}

fn payload_of(kind: Kind, example: &Value) -> Option<Value> {
    match kind {
        Kind::Shared => Some(example.clone()),
        _ => example.get("payload").filter(|p| p.is_object()).cloned(),
    }
}

fn with_payload(kind: Kind, example: &Value, payload: Value) -> Value {
    match kind {
        Kind::Shared => payload,
        _ => {
            let mut m = example.clone();
            m["payload"] = payload;
            m
        }
    }
}

// doc/shared_libraries.md#contract-tests
fn variants_of(name: &str, kind: Kind, example: &Value) -> Vec<(String, Value)> {
    let mut out = vec![("example".to_string(), example.clone())];
    let Some(payload) = payload_of(kind, example) else { return out };
    let s = payload_schema(name, kind);
    let required: BTreeSet<&str> = s["required"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
    for (field, fs) in s["properties"].as_object().into_iter().flatten() {
        let mut add = |desc: String, value: Option<Value>| {
            let mut p = payload.clone();
            match value {
                Some(v) => p[field] = v,
                None => {
                    p.as_object_mut().unwrap().remove(field);
                }
            }
            out.push((desc, with_payload(kind, example, p)));
        };
        if !required.contains(field.as_str()) {
            add(format!("{field} left out"), None);
        }
        if allows_null(fs) {
            add(format!("{field} null"), Some(Value::Null));
        }
        for v in enum_values(fs) {
            add(format!("{field} = {v}"), Some(v));
        }
    }
    out
}


// LIB-CON-01
#[test]
fn lib_con_01_every_schema_has_an_example() {
    let examples = examples();
    let missing: Vec<String> =
        schema_names().into_iter().filter(|n| !examples.contains_key(n)).map(|n| format!("{n}: no example")).collect();
    report(missing);
}

// LIB-CON-02
#[test]
fn lib_con_02_valid_examples_are_valid() {
    let names = schema_names();
    let mut failures = Vec::new();
    for (name, example) in examples() {
        if !names.contains(&name) {
            failures.push(format!("{name}: example without schema"));
            continue;
        }
        failures.extend(errors(&name, &example).into_iter().map(|e| format!("{name}: {e}")));
    }
    // doc/shared_libraries.md#target-types
    for p in json_files(&examples_dir().join("target_types")) {
        let v = read_json(&p);
        if v["valid"] == true {
            failures.extend(errors("common/target_type", &v["target_type"]).into_iter().map(|e| format!("{}: {e}", p.display())));
        }
    }
    report(failures);
}

// LIB-CON-03
#[test]
fn lib_con_03_invalid_examples_are_invalid() {
    let mut failures = Vec::new();
    let all = invalid_examples();
    assert!(!all.is_empty());
    for inv in all {
        let errs = errors(&inv.schema, &inv.message);
        let field = inv.expected_error.split(':').next().unwrap().trim();
        if errs.is_empty() {
            failures.push(format!("{}: accepted", inv.file));
        } else if !errs.iter().any(|e| e.contains(field)) {
            failures.push(format!("{}: expected an error about {field}, got {errs:?}", inv.file));
        }
    }
    report(failures);
}

// LIB-CON-04
#[test]
fn lib_con_04_round_trip() {
    let reg = registry();
    let mut failures = Vec::new();
    for (name, example) in examples() {
        let Some(entry) = reg.get(&name) else { continue };
        match (entry.round_trip)(&example) {
            Ok(back) if json_eq(&back, &example) => {}
            Ok(back) => failures.push(format!("{name}: round trip differs\n  example: {example}\n  back:    {back}")),
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }
    report(failures);
}

// LIB-CON-05
#[test]
fn lib_con_05_produced_messages_are_valid() {
    let reg = registry();
    let mut failures = Vec::new();
    let mut checked = 0;
    for (name, example) in examples() {
        let Some(entry) = reg.get(&name) else { continue };
        for (desc, variant) in variants_of(&name, entry.kind, &example) {
            if !errors(&name, &variant).is_empty() {
                continue;
            }
            checked += 1;
            match (entry.produce)(&variant) {
                Ok(produced) => {
                    failures.extend(errors(&name, &produced).into_iter().map(|e| format!("{name} ({desc}): {e}")))
                }
                Err(e) => failures.push(format!("{name} ({desc}): valid message rejected: {e}")),
            }
        }
        // doc/interfaces.md#error-codes
        if entry.kind == Kind::Response {
            let req: RequestEnvelope = serde_json::from_value(json!({
                "type": name.rsplit('/').next().unwrap().trim_end_matches("_response"),
                "version": 1, "request_id": example["request_id"], "payload": {}
            }))
            .unwrap();
            for code in all_error_codes() {
                for details in [None, Some(json!({ "setting": "export_dir" }))] {
                    let mut e = ServiceError::new(code, "message");
                    e.details = details;
                    let produced = serde_json::to_value(req.error_response(&e)).unwrap();
                    checked += 1;
                    failures.extend(errors(&name, &produced).into_iter().map(|e| format!("{name} ({code:?}): {e}")));
                }
            }
        }
    }
    assert!(checked > 500, "only {checked} messages checked");
    report(failures);
}

fn all_error_codes() -> Vec<ErrorCode> {
    let s = schema("common/error_code");
    s["enum"].as_array().unwrap().iter().map(|c| serde_json::from_value(c.clone()).unwrap()).collect()
}

// LIB-CON-06
#[test]
fn lib_con_06_invalid_input_is_rejected() {
    let reg = registry();
    let mut failures = Vec::new();
    for inv in invalid_examples() {
        let Some(entry) = reg.get(&inv.schema) else {
            failures.push(format!("{}: no Rust type for {}", inv.file, inv.schema));
            continue;
        };
        let result = std::panic::catch_unwind(|| (entry.receive)(&inv.message));
        match result {
            Err(_) => failures.push(format!("{}: panic", inv.file)),
            Ok(Ok(v)) => failures.push(format!("{}: accepted as {v}", inv.file)),
            Ok(Err(_)) => {}
        }
    }
    report(failures);
}

// LIB-CON-07
#[test]
fn lib_con_07_completeness_rust() {
    let reg = registry();
    let names = schema_names();
    let mut failures = Vec::new();
    for n in &names {
        if !reg.contains_key(n) {
            failures.push(format!("schema {n} has no Rust type"));
        }
    }
    for n in reg.keys() {
        if !names.contains(n) {
            failures.push(format!("Rust type for {n} has no schema"));
        }
    }
    let enums: Vec<(&str, Vec<String>)> = vec![
        (analyzer::PROGRAM, variants::<analyzer::AnalyzerRequest>()),
        (camera_reader::PROGRAM, variants::<camera_reader::CameraReaderRequest>()),
        (camera_reader::PROGRAM, variants::<camera_reader::CameraReaderEvent>()),
        (db::PROGRAM, variants::<db::DatabaseHandlerRequest>()),
        (db::PROGRAM, variants::<db::DatabaseHandlerEvent>()),
        (exporter::PROGRAM, variants::<exporter::ExporterRequest>()),
        (ip::PROGRAM, variants::<ip::ImageProcessorRequest>()),
        (ip::PROGRAM, variants::<ip::ImageProcessorEvent>()),
        (importer::PROGRAM, variants::<importer::ImporterRequest>()),
        (launcher::PROGRAM, variants::<launcher::LauncherRequest>()),
        (launcher::PROGRAM, variants::<launcher::LauncherEvent>()),
        (sm::PROGRAM, variants::<sm::SessionManagerRequest>()),
        (sm::PROGRAM, variants::<sm::SessionManagerEvent>()),
        (statistics::PROGRAM, variants::<statistics::StatisticsRequest>()),
    ];
    for (program, vs) in enums {
        assert!(!vs.is_empty(), "{program}: no variants found");
        for v in vs {
            let name = if v == "ping" { "common/ping".to_string() } else { format!("{program}/{v}") };
            if !names.contains(&name) {
                failures.push(format!("{program}: variant {v} has no schema {name}"));
            }
        }
    }
    let src = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/model.rs")).unwrap();
    for line in src.lines() {
        let Some(rest) = line.trim().strip_prefix("pub struct ") else { continue };
        let ty: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
        // doc/interfaces.md#import_session
        if ty == "ImportedSession" {
            continue;
        }
        let name = format!("common/{}", snake(&ty));
        if !reg.contains_key(&name) {
            failures.push(format!("model::{ty} has no schema {name}"));
        }
    }
    report(failures);
}

fn snake(camel: &str) -> String {
    let mut out = String::new();
    for (i, c) in camel.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

// LIB-CON-08
#[test]
#[ignore = "C++ test: doc/testing/shared_libraries.md#contract-schemas-and-examples-contract"]
fn lib_con_08_completeness_c() {}

// LIB-CON-09
#[test]
#[ignore = "needs targetscope_common_cpp: doc/testing/shared_libraries.md#contract-schemas-and-examples-contract"]
fn lib_con_09_rust_and_c_agree() {}

// LIB-CON-10
#[test]
fn lib_con_10_missing_and_null_optional_fields() {
    let reg = registry();
    let mut failures = Vec::new();
    for (name, example) in examples() {
        // doc/interfaces.md#session_manager
        if name == "session_manager/session_update_setup" {
            continue;
        }
        let Some(entry) = reg.get(&name) else { continue };
        let Some(payload) = payload_of(entry.kind, &example) else { continue };
        let s = payload_schema(&name, entry.kind);
        let required: BTreeSet<&str> =
            s["required"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
        for (field, fs) in s["properties"].as_object().into_iter().flatten() {
            if !allows_null(fs) {
                continue;
            }
            let mut missing = payload.clone();
            missing.as_object_mut().unwrap().remove(field);
            let mut null = payload.clone();
            null[field] = Value::Null;
            let a = (entry.round_trip)(&with_payload(entry.kind, &example, missing));
            let b = (entry.round_trip)(&with_payload(entry.kind, &example, null));
            match (a, b) {
                (Ok(a), Ok(b)) if json_eq(&a, &b) => {}
                (Err(_), Ok(_)) if required.contains(field.as_str()) => {}
                (a, b) => failures.push(format!("{name}.{field}: missing gives {a:?}, null gives {b:?}")),
            }
        }
    }
    report(failures);
}

// LIB-CON-11
#[test]
fn lib_con_11_field_names() {
    fn check(v: &Value, file: &str, failures: &mut Vec<String>) {
        match v {
            Value::Object(m) => {
                if let Some(Value::Object(props)) = m.get("properties") {
                    for k in props.keys() {
                        let ok = k.chars().next().is_some_and(|c| c.is_ascii_lowercase())
                            && k.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
                        if !ok {
                            failures.push(format!("{file}: {k}"));
                        }
                    }
                }
                for x in m.values() {
                    check(x, file, failures);
                }
            }
            Value::Array(a) => a.iter().for_each(|x| check(x, file, failures)),
            _ => {}
        }
    }
    let mut failures = Vec::new();
    for name in schema_names() {
        check(&schema(&name), &name, &mut failures);
    }
    report(failures);
}
