// doc/programs/session_manager.md#saving

use targetscope_common::bus::BusError;
use targetscope_common::failpoints;
use targetscope_common::messages::database_handler::CreateSession;
use targetscope_common::messages::{ErrorCode, ServiceError};
use targetscope_common::model::{InputMode, NewSession, SessionShot};
use uuid::Uuid;

use crate::actor::{err, require_records, Saved, SessionActor};
use crate::state::StoredSession;

pub async fn save(actor: &mut SessionActor, s: &mut StoredSession) -> Result<Saved, ServiceError> {
    if s.session.shots.is_empty() {
        return Err(ServiceError::validation("at least one shot is required"));
    }
    require_records(s)?;
    if s.save_request_id.is_none() {
        s.save_request_id = Some(Uuid::new_v4());
        actor.persist(s.clone()).await.map_err(err)?;
    }
    let request_id = s.save_request_id.expect("set above");
    if actor.reread_records(s).await? {
        s.shots_changed();
    }
    let (_, _) = require_records(s)?;
    if s.session.analysis_outdated || s.session.analysis.is_none() {
        let result = actor.analyze_now(s).await?;
        s.session.analysis = Some(result);
        s.session.analysis_outdated = false;
    }
    let analysis = s.session.analysis.clone().expect("analysis set");
    let mut warnings = s.save_warnings.clone();
    let mut images: Vec<_> = s.session.paper_images.iter().filter(|i| i.paper_no <= s.session.paper_no).cloned().collect();
    loop {
        let new = NewSession {
            score: analysis.score,
            grouping_score: analysis.grouping_score,
            distance_m: s.session.setup.distance_m,
            target_type_id: s.session.setup.target_type_id.expect("checked"),
            shooter_id: s.session.setup.shooter_id,
            device_id: s.session.setup.device_id.expect("checked"),
            single_handed: s.session.setup.single_handed,
            supported: s.session.setup.supported,
            shot_count: s.session.shots.len() as i64,
            duration_s: (s.mode() != InputMode::Photo).then_some(s.session.duration_s),
            notes: s.session.notes.clone(),
            paper_count: s.session.paper_no,
            shots: s
                .session
                .shots
                .iter()
                .map(|x| SessionShot { angle_turns: x.angle_turns, radius_mm: x.radius_mm, paper_no: x.paper_no })
                .collect(),
            images: images.clone(),
        };
        let result = actor.bus.request_with_id(request_id, CreateSession(new)).await;
        failpoints::hit("sm.after_create_session_sent");
        match result {
            Ok(r) => {
                *s = StoredSession::idle();
                return Ok(Saved { session_id: r.id, warnings });
            }
            Err(BusError::Remote(e)) if e.code == ErrorCode::NotFound => {
                let missing = e.details.as_ref().and_then(|d| d["staged_file"].as_str()).map(str::to_string);
                let Some(pos) = missing.and_then(|m| images.iter().position(|i| i.staged_file == m)) else {
                    return Err(e.into());
                };
                let gone = images.remove(pos);
                warnings.push(format!("paper {} saved without image: the staged image expired", gone.paper_no));
            }
            Err(e) => return Err(e.into()),
        }
    }
}
