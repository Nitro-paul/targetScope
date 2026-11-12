// doc/programs/exporter.md#png-target-plot

use ab_glyph::{point, Font, FontRef, PxScale, ScaleFont};
use targetscope_common::clock::rfc3339_local;
use targetscope_common::geometry::Point;
use targetscope_common::model::{AnalysisResult, Device, Session, Shooter, TargetType};
use targetscope_common::targets::generate_rings;
use tiny_skia::{Color, FillRule, Mask, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

pub const LONG_SIDE_PX: f32 = 1600.0;
pub const HEADER_PX: f32 = 170.0;
const TEXT_PX: f32 = 30.0;
const LABEL_PX: f32 = 22.0;

pub static FONT: &[u8] = include_bytes!("../../assets/DejaVuSans.ttf");

struct Canvas {
    pixmap: Pixmap,
    font: FontRef<'static>,
    scale: f32,
    center: (f32, f32),
}

struct TextLayer {
    mask: Mask,
}

impl Canvas {
    fn paper(&self, x_mm: f64, y_mm: f64) -> (f32, f32) {
        (x_mm as f32 * self.scale, HEADER_PX + y_mm as f32 * self.scale)
    }

    fn shot(&self, p: Point) -> (f32, f32) {
        (self.center.0 + p.x as f32 * self.scale, self.center.1 - p.y as f32 * self.scale)
    }

    fn circle(&mut self, c: (f32, f32), r: f32, fill: Option<Color>, stroke: Option<(Color, f32)>) {
        let Some(path) = PathBuilder::from_circle(c.0, c.1, r.max(0.5)) else { return };
        if let Some(color) = fill {
            self.pixmap.fill_path(&path, &paint(color), FillRule::Winding, Transform::identity(), None);
        }
        if let Some((color, width)) = stroke {
            let s = Stroke { width, ..Default::default() };
            self.pixmap.stroke_path(&path, &paint(color), &s, Transform::identity(), None);
        }
    }

    fn line(&mut self, a: (f32, f32), b: (f32, f32), color: Color, width: f32) {
        let mut pb = PathBuilder::new();
        pb.move_to(a.0, a.1);
        pb.line_to(b.0, b.1);
        if let Some(path) = pb.finish() {
            let s = Stroke { width, ..Default::default() };
            self.pixmap.stroke_path(&path, &paint(color), &s, Transform::identity(), None);
        }
    }

    fn text_width(&self, text: &str, size: f32) -> f32 {
        let scaled = self.font.as_scaled(PxScale::from(size));
        text.chars().map(|c| scaled.h_advance(self.font.glyph_id(c))).sum()
    }

    fn text(&self, layer: &mut TextLayer, text: &str, x: f32, baseline: f32, size: f32) {
        let scale = PxScale::from(size);
        let scaled = self.font.as_scaled(scale);
        let (w, h) = (layer.mask.width() as i32, layer.mask.height() as i32);
        let mut caret = x;
        for c in text.chars() {
            let id = self.font.glyph_id(c);
            let glyph = id.with_scale_and_position(scale, point(caret, baseline));
            caret += scaled.h_advance(id);
            let Some(outline) = self.font.outline_glyph(glyph) else { continue };
            let bounds = outline.px_bounds();
            let data = layer.mask.data_mut();
            outline.draw(|gx, gy, coverage| {
                let (px, py) = (bounds.min.x as i32 + gx as i32, bounds.min.y as i32 + gy as i32);
                if px >= 0 && py >= 0 && px < w && py < h {
                    let i = (py * w + px) as usize;
                    data[i] = data[i].max((coverage.clamp(0.0, 1.0) * 255.0) as u8);
                }
            });
        }
    }

    fn layer(&self) -> TextLayer {
        TextLayer { mask: Mask::new(self.pixmap.width(), self.pixmap.height()).expect("mask size") }
    }

    fn paint_layer(&mut self, layer: &TextLayer, color: Color) {
        let rect = Rect::from_xywh(0.0, 0.0, self.pixmap.width() as f32, self.pixmap.height() as f32).expect("rect");
        self.pixmap.fill_rect(rect, &paint(color), Transform::identity(), Some(&layer.mask));
    }
}

fn paint(color: Color) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(color);
    p.anti_alias = true;
    p
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba8(r, g, b, a)
}

// doc/programs/exporter.md#png-target-plot
pub fn shot_labels(session: &Session) -> Vec<String> {
    let mut count = std::collections::HashMap::new();
    session
        .shots
        .iter()
        .map(|s| {
            let n = count.entry(s.paper_no).or_insert(0);
            *n += 1;
            if session.paper_count > 1 { format!("{}.{}", s.paper_no, n) } else { n.to_string() }
        })
        .collect()
}

pub fn render(
    session: &Session,
    analysis: &AnalysisResult,
    target: &TargetType,
    device: &Device,
    shooter: Option<&Shooter>,
) -> Result<Vec<u8>, String> {
    let scale = LONG_SIDE_PX / target.paper_width_mm.max(target.paper_height_mm) as f32;
    let (w, h) = ((target.paper_width_mm as f32 * scale).round(), (target.paper_height_mm as f32 * scale).round());
    let mut pixmap = Pixmap::new(w as u32, (h + HEADER_PX) as u32).ok_or("image size")?;
    pixmap.fill(Color::WHITE);
    let font = FontRef::try_from_slice(FONT).map_err(|e| e.to_string())?;
    let mut c = Canvas { pixmap, font, scale, center: (0.0, 0.0) };
    c.center = c.paper(target.center_x_mm, target.center_y_mm);
    let mut black_text = c.layer();
    let mut white_text = c.layer();

    let header = [
        format!(
            "{}   {}   {}",
            rfc3339_local(session.saved_at),
            shooter.map(|s| s.name.as_str()).unwrap_or("no shooter"),
            device.name
        ),
        format!(
            "{}   {}",
            session.distance_m.map(|d| format!("{d} m")).unwrap_or_else(|| "distance not set".into()),
            target.name
        ),
        format!(
            "score {}   grouping score {}   shots {}   papers {}",
            session.score, session.grouping_score, session.shot_count, session.paper_count
        ),
    ];
    for (i, line) in header.iter().enumerate() {
        c.text(&mut black_text, line, 20.0, 45.0 + i as f32 * 48.0, TEXT_PX);
    }

    let paper = Rect::from_xywh(0.5, HEADER_PX + 0.5, w - 1.0, h - 1.0).ok_or("paper size")?;
    c.pixmap.stroke_path(
        &PathBuilder::from_rect(paper),
        &paint(rgba(120, 120, 120, 255)),
        &Stroke { width: 2.0, ..Default::default() },
        Transform::identity(),
        None,
    );
    c.circle(c.center, target.black_radius_mm as f32 * scale, Some(Color::BLACK), None);
    let rings = generate_rings(target);
    let mut inner = 0.0f32;
    for ring in &rings {
        let r = ring.radius_mm as f32 * scale;
        let in_black = ring.radius_mm <= target.black_radius_mm;
        let line = if in_black { Color::WHITE } else { Color::BLACK };
        c.circle(c.center, r, None, Some((line, 2.0)));
        let label = ring.score.to_string();
        let band = r - inner;
        if band >= LABEL_PX * 0.8 {
            let x = c.center.0 + (inner + r) / 2.0 - c.text_width(&label, LABEL_PX) / 2.0;
            let layer = if in_black { &mut white_text } else { &mut black_text };
            c.text(layer, &label, x, c.center.1 + LABEL_PX * 0.35, LABEL_PX);
        }
        inner = r;
    }

    let bullet_r = (device.bullet_diameter_mm / 2.0) as f32 * scale;
    for (shot, label) in session.shots.iter().zip(shot_labels(session)) {
        let p = c.shot(Point::from(shot.shot()));
        c.circle(p, bullet_r, Some(rgba(30, 120, 255, 170)), Some((rgba(0, 60, 160, 255), 2.0)));
        let (lx, ly) = (p.0 + bullet_r + 4.0, p.1 - bullet_r - 4.0);
        let from_center = ((lx - c.center.0).powi(2) + (ly - c.center.1).powi(2)).sqrt();
        let layer = if from_center <= target.black_radius_mm as f32 * scale { &mut white_text } else { &mut black_text };
        c.text(layer, &label, lx, ly, LABEL_PX);
    }

    if let Some(m) = analysis.mean_point {
        let p = c.shot(Point { x: m.x_mm, y: m.y_mm });
        let red = rgba(220, 30, 30, 255);
        c.line((p.0 - 14.0, p.1), (p.0 + 14.0, p.1), red, 4.0);
        c.line((p.0, p.1 - 14.0), (p.0, p.1 + 14.0), red, 4.0);
    }
    let shift = Point { x: analysis.grouping_shift.x_mm, y: analysis.grouping_shift.y_mm };
    if shift.length() > 0.0 {
        let green = rgba(20, 170, 60, 255);
        let tip = c.shot(shift);
        c.line(c.center, tip, green, 4.0);
        let (dx, dy) = (tip.0 - c.center.0, tip.1 - c.center.1);
        let len = (dx * dx + dy * dy).sqrt().max(1e-3);
        let (ux, uy) = (dx / len, dy / len);
        let head = 18.0;
        for side in [-1.0f32, 1.0] {
            let wing = (tip.0 - head * ux + side * head * 0.5 * uy, tip.1 - head * uy - side * head * 0.5 * ux);
            c.line(tip, wing, green, 4.0);
        }
    }

    c.paint_layer(&black_text, Color::BLACK);
    c.paint_layer(&white_text, Color::WHITE);
    c.pixmap.encode_png().map_err(|e| e.to_string())
}
