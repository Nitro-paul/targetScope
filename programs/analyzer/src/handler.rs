// doc/programs/analyzer.md#interface

use async_trait::async_trait;
use serde_json::Value;
use targetscope_common::bus::RequestHandler;
use targetscope_common::messages::analyzer::{AnalyzeShots, AnalyzerRequest};
use targetscope_common::messages::ServiceError;
use targetscope_common::targets::validate_target_type;
use uuid::Uuid;

use crate::scoring::analyze;

pub struct AnalyzerHandler;

#[async_trait]
impl RequestHandler<AnalyzerRequest> for AnalyzerHandler {
    async fn handle(&self, _request_id: Uuid, req: AnalyzerRequest) -> Result<Value, ServiceError> {
        match req {
            AnalyzerRequest::AnalyzeShots(r) => {
                validate(&r)?;
                let result = tokio::task::spawn_blocking(move || analyze(&r.shots, &r.target_type, r.bullet_diameter_mm))
                    .await
                    .map_err(|e| ServiceError::new(targetscope_common::messages::ErrorCode::ProcessingFailed, e.to_string()))?;
                Ok(serde_json::to_value(result).expect("serializable"))
            }
            AnalyzerRequest::Ping(_) => Ok(Value::Null),
        }
    }
}

// doc/programs/analyzer.md#interface
pub fn validate(r: &AnalyzeShots) -> Result<(), ServiceError> {
    validate_target_type(&r.target_type).map_err(ServiceError::validation)?;
    if !(r.bullet_diameter_mm.is_finite() && r.bullet_diameter_mm > 0.0) {
        return Err(ServiceError::validation("bullet_diameter_mm > 0"));
    }
    if let Some(i) = r.shots.iter().position(|s| !s.is_valid()) {
        return Err(ServiceError::validation(format!("shot {i}: 0 <= angle_turns < 1 and radius_mm >= 0")));
    }
    Ok(())
}
