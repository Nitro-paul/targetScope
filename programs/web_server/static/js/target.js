// doc/programs/web_server.md#coordinate-conversion-in-the-ui
// doc/interfaces.md#targettype

const SVG = 'http://www.w3.org/2000/svg';

export function generateRings(t) {
  const anchors = [
    [t.inner_white_score, t.inner_white_radius_mm],
    [t.black_score, t.black_radius_mm],
    [t.outer_score, t.outer_radius_mm],
  ];
  const radius = (s) => {
    const exact = anchors.find(([score]) => score === s);
    if (exact) return exact[1];
    const [a, b] = s > anchors[1][0] ? [anchors[0], anchors[1]] : [anchors[1], anchors[2]];
    return a[1] + ((a[0] - s) * (b[1] - a[1])) / (a[0] - b[0]);
  };
  const rings = [];
  for (let s = t.inner_white_score; s >= t.outer_score && s >= 1; s--) rings.push({ score: s, radius_mm: radius(s) });
  return rings;
}

// doc/shared_libraries.md#target-types
export function validateTargetType(t) {
  const sizes = [t.paper_width_mm, t.paper_height_mm, t.center_x_mm, t.center_y_mm, t.inner_white_radius_mm, t.black_radius_mm, t.outer_radius_mm];
  if (sizes.some((v) => typeof v !== 'number' || !Number.isFinite(v))) return 'all sizes are finite numbers';
  const rules = [
    [t.paper_width_mm > 0 && t.paper_height_mm > 0, 'paper size > 0'],
    [t.inner_white_score > t.black_score, 'inner_white_score > black_score'],
    [t.black_score > t.outer_score, 'black_score > outer_score'],
    [t.outer_score >= 1, 'outer_score >= 1'],
    [t.inner_white_radius_mm > 0, 'inner_white_radius_mm > 0'],
    [t.inner_white_radius_mm < t.black_radius_mm, 'inner_white_radius_mm < black_radius_mm'],
    [t.black_radius_mm < t.outer_radius_mm, 'black_radius_mm < outer_radius_mm'],
    [t.center_x_mm - t.outer_radius_mm >= 0, 'outermost ring on the paper (left edge)'],
    [t.center_x_mm + t.outer_radius_mm <= t.paper_width_mm, 'outermost ring on the paper (right edge)'],
    [t.center_y_mm - t.outer_radius_mm >= 0, 'outermost ring on the paper (top edge)'],
    [t.center_y_mm + t.outer_radius_mm <= t.paper_height_mm, 'outermost ring on the paper (bottom edge)'],
  ];
  const broken = rules.find(([ok]) => !ok);
  return broken ? broken[1] : null;
}

// doc/interfaces.md#shot
export function toPolar(x, y) {
  const radius_mm = Math.hypot(x, y);
  if (radius_mm === 0) return { angle_turns: 0, radius_mm: 0 };
  let angle_turns = Math.atan2(x, y) / (2 * Math.PI);
  if (angle_turns < 0) angle_turns += 1;
  if (angle_turns >= 1) angle_turns = 0;
  return { angle_turns, radius_mm };
}

export function toCartesian(shot) {
  const a = 2 * Math.PI * shot.angle_turns;
  return { x: shot.radius_mm * Math.sin(a), y: shot.radius_mm * Math.cos(a) };
}

// doc/programs/web_server.md#coordinate-conversion-in-the-ui
export function paperToShot(t, xMm, yMm) {
  return toPolar(xMm - t.center_x_mm, t.center_y_mm - yMm);
}

export function shotToPaper(t, shot) {
  const p = toCartesian(shot);
  return { x: t.center_x_mm + p.x, y: t.center_y_mm - p.y };
}

// doc/interfaces.md#currentshot
export function shotLabel(paperCount, paperNo, n) {
  return paperCount > 1 ? `${paperNo}.${n}` : String(n);
}

export function shotLabels(shots, paperCount) {
  const counts = new Map();
  return shots.map((s) => {
    const n = (counts.get(s.paper_no) || 0) + 1;
    counts.set(s.paper_no, n);
    return shotLabel(paperCount, s.paper_no, n);
  });
}

function el(name, attrs = {}, parent = null) {
  const e = document.createElementNS(SVG, name);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
  if (parent) parent.appendChild(e);
  return e;
}

// doc/programs/web_server.md#coordinate-conversion-in-the-ui
export function svgPoint(svg, event) {
  const pt = svg.createSVGPoint();
  pt.x = event.clientX;
  pt.y = event.clientY;
  return pt.matrixTransform(svg.getScreenCTM().inverse());
}

// doc/programs/web_server.md#session-view-per-state
export function renderTarget(svg, t, shots = [], opts = {}) {
  const { bulletDiameterMm = 4.5, paperCount = 1, image = null, selected = null, highlightLow = true, shift = null, mean = null } = opts;
  svg.replaceChildren();
  svg.setAttribute('viewBox', `0 0 ${t.paper_width_mm} ${t.paper_height_mm}`);
  svg.classList.add('target');
  el('rect', { x: 0, y: 0, width: t.paper_width_mm, height: t.paper_height_mm, class: 'paper' }, svg);
  if (image) el('image', { href: image, x: 0, y: 0, width: t.paper_width_mm, height: t.paper_height_mm, preserveAspectRatio: 'none', opacity: 0.85 }, svg);
  const cx = t.center_x_mm;
  const cy = t.center_y_mm;
  if (!image) el('circle', { cx, cy, r: t.black_radius_mm, class: 'black' }, svg);
  const rings = generateRings(t);
  let inner = 0;
  for (const ring of rings) {
    const inBlack = ring.radius_mm <= t.black_radius_mm && !image;
    el('circle', { cx, cy, r: ring.radius_mm, class: inBlack ? 'ring ring-light' : 'ring' }, svg);
    const band = ring.radius_mm - inner;
    if (band > 2.5) {
      const label = el('text', { x: cx + (inner + ring.radius_mm) / 2, y: cy + 1, class: inBlack ? 'ring-label ring-light' : 'ring-label' }, svg);
      label.textContent = ring.score;
    }
    inner = ring.radius_mm;
  }
  if (mean) {
    const m = { x: cx + mean.x_mm, y: cy - mean.y_mm };
    el('path', { d: `M${m.x - 3},${m.y} h6 M${m.x},${m.y - 3} v6`, class: 'mean' }, svg);
  }
  if (shift && (shift.x_mm || shift.y_mm)) {
    el('line', { x1: cx, y1: cy, x2: cx + shift.x_mm, y2: cy - shift.y_mm, class: 'shift' }, svg);
  }
  const labels = shotLabels(shots, paperCount);
  shots.forEach((s, i) => {
    const p = shotToPaper(t, s);
    const low = highlightLow && s.confidence != null && s.confidence < 0.5;
    const cls = ['shot', i === selected ? 'selected' : '', low ? 'low-confidence' : ''].join(' ').trim();
    const c = el('circle', { cx: p.x, cy: p.y, r: bulletDiameterMm / 2, class: cls, 'data-index': i }, svg);
    c.dataset.index = i;
    const text = el('text', { x: p.x + bulletDiameterMm / 2 + 0.8, y: p.y - bulletDiameterMm / 2, class: 'shot-label' }, svg);
    text.textContent = labels[i];
  });
  return svg;
}
