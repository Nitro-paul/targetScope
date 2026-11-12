//! PNG target plot `<prefix>_session_<id>.png`: paper to scale (long side 1600 px) plus header:
//! rings with scores, shots as circles with the bullet diameter and their numbers, mean point
//! as a cross, grouping shift as an arrow. tiny-skia + ab_glyph with the embedded font.

use targetscope_common::model::{AnalysisResult, Device, Session, Shooter, TargetType};

pub const LONG_SIDE_PX: u32 = 1600;

/// Embedded font (assets/DejaVuSans.ttf).
// TODO: pub static FONT: &[u8] = include_bytes!("../../assets/DejaVuSans.ttf");

pub fn render(
    session: &Session,
    analysis: &AnalysisResult,
    target: &TargetType,
    device: &Device,
    shooter: Option<&Shooter>,
) -> Vec<u8> {
    let _ = (session, analysis, target, device, shooter);
    todo!()
}
