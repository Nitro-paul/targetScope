// doc/programs/web_server.md#session-view-per-state

import * as api from '../api.js';
import { on, onReload } from '../events.js';
import { createOverlay, displayedToImage } from '../overlay.js';
import { paperToShot, renderTarget, shotLabels, svgPoint, shotToPaper } from '../target.js';
import { confirmDialog, dialog, fmtDuration, fmtTime, h, run, select, showError } from '../ui.js';

let root = null;
let session = null;
let records = { shooters: [], devices: [], targets: [] };
let selected = null;
let reviewPaper = null;
let unsubscribe = [];
let timer = null;

export async function render(el) {
  root = el;
  unsubscribe.push(on('session_changed', (p) => show(p.session)));
  unsubscribe.push(on('data_changed', () => loadRecords().then(() => show(session))));
  unsubscribe.push(onReload(refresh));
  timer = setInterval(tick, 1000);
  await loadRecords();
  await refresh();
}

export function leave() {
  unsubscribe.forEach((u) => u());
  unsubscribe = [];
  clearInterval(timer);
}

async function refresh() {
  await run(async () => show((await api.get('/api/session')).session));
}

async function loadRecords() {
  const [s, d, t] = await Promise.all([api.get('/api/shooters'), api.get('/api/devices'), api.get('/api/target-types')]);
  records = { shooters: s.shooters, devices: d.devices, targets: t.target_types };
}

const target = () => records.targets.find((t) => t.id === session?.setup.target_type_id);
const device = () => records.devices.find((d) => d.id === session?.setup.device_id);

async function act(method, path, body) {
  return run(async () => {
    const r = await api.request(method, path, body);
    if (r?.session) show(r.session);
    return r;
  });
}

let shownAt = Date.now();

function tick() {
  const el = document.getElementById('running-time');
  if (!el || !session) return;
  const extra = session.state === 'running' ? Math.floor((Date.now() - shownAt) / 1000) : 0;
  el.textContent = fmtDuration(session.duration_s + extra);
}

function show(s) {
  if (!root || !s) return;
  session = s;
  shownAt = Date.now();
  const views = { idle, setup, running: live, paused: live, review, recovered };
  root.replaceChildren(h('section', { class: `session state-${s.state}` }, views[s.state]()));
}

// doc/programs/web_server.md#session-view-per-state
function idle() {
  const start = (mode) => act('POST', '/api/session/setup', { input_mode: mode });
  return h('div', { class: 'card' }, h('h1', {}, 'New session'), h('p', {}, 'How are the shots entered?'),
    h('div', { class: 'buttons' },
      h('button', { class: 'primary', onclick: () => start('camera') }, 'Camera'),
      h('button', { onclick: () => start('manual') }, 'Manual (click on the target)'),
      h('button', { onclick: () => start('photo') }, 'Photo of the shot target')));
}

function active(list) {
  return list.filter((x) => !x.retired);
}

function setupForm() {
  const s = session.setup;
  const update = (patch) => act('PATCH', '/api/session/setup', patch);
  const opts = (list) => active(list).map((x) => ({ value: x.id, label: x.name }));
  const newShooter = h('button', { type: 'button', onclick: async () => {
    const input = h('input', { placeholder: 'Name' });
    if (!(await dialog('New shooter', h('label', { class: 'field' }, 'Name', input), [['Cancel', false], ['Create', true, 'primary']]))) return;
    const r = await run(() => api.post('/api/shooters', { name: input.value, retired: false }));
    if (r) {
      await loadRecords();
      update({ shooter_id: r.id });
    }
  } }, 'New shooter');
  const num = (v) => (v === '' || v == null ? null : Number(v));
  return h('div', { class: 'form' },
    h('label', { class: 'field' }, 'Shooter', select(opts(records.shooters), s.shooter_id, (v) => update({ shooter_id: num(v) }), { empty: 'no shooter' }), newShooter),
    h('label', { class: 'field' }, 'Device', select(opts(records.devices), s.device_id, (v) => update({ device_id: num(v) }), { empty: 'choose…' })),
    h('label', { class: 'field' }, 'Distance (m)', h('input', { type: 'number', min: 0, value: s.distance_m ?? '', onchange: (e) => update({ distance_m: num(e.target.value) }) })),
    h('label', { class: 'field' }, 'Target type', select(opts(records.targets), s.target_type_id, (v) => update({ target_type_id: num(v) }), { empty: 'choose…' })),
    h('label', { class: 'field check' }, h('input', { type: 'checkbox', checked: s.single_handed, onchange: (e) => update({ single_handed: e.target.checked }) }), ' single-handed'),
    h('label', { class: 'field check' }, h('input', { type: 'checkbox', checked: s.supported, onchange: (e) => update({ supported: e.target.checked }) }), ' supported'));
}

function calibrationStatus() {
  const c = session.calibration;
  if (!c) return h('p', { class: 'muted' }, 'Choose device and target type to start the live view.');
  const text = { calibrated: 'Paper found', lost: 'Paper lost', not_calibrated: 'Paper not found' }[c.state] || c.state;
  return h('p', { class: `calibration ${c.state}` }, text, c.message ? ` (${c.message})` : '', c.method ? ` · ${c.method}` : '');
}

// doc/programs/web_server.md#coordinate-conversion-in-the-ui
function manualCalibration() {
  const img = h('img', { src: `/stream/raw.mjpg?${Date.now()}`, class: 'feed clickable', alt: 'Camera image' });
  const clicks = [];
  const info = h('p', {}, 'Click the four paper corners: top left, top right, bottom right, bottom left.');
  const marks = h('div', { class: 'marks' });
  img.addEventListener('click', (e) => {
    const r = img.getBoundingClientRect();
    const raw = displayedToImage({ x: e.clientX - r.left, y: e.clientY - r.top }, img.clientWidth, img.naturalWidth || img.clientWidth);
    clicks.push(raw);
    marks.append(h('span', { class: 'mark', style: `left:${e.clientX - r.left}px;top:${e.clientY - r.top}px` }));
    if (clicks.length === 4) {
      const [tl, tr, br, bl] = clicks;
      act('POST', '/api/session/calibrate', { top_left: tl, top_right: tr, bottom_right: br, bottom_left: bl });
    }
  });
  return h('div', { class: 'calibrate' }, info, h('div', { class: 'feed-box' }, img, marks));
}

function liveView() {
  const box = h('div', { class: 'live' }, h('img', { src: `/stream/live.mjpg?${Date.now()}`, class: 'feed', alt: 'Live view' }), calibrationStatus());
  if (session.calibration && session.calibration.state !== 'calibrated') {
    box.append(h('button', { onclick: () => box.replaceChildren(manualCalibration()) }, 'Calibrate manually'));
  }
  return box;
}

// doc/programs/web_server.md#session-view-per-state
async function uploadPhoto(file, corners) {
  const form = new FormData();
  form.append('photo', file);
  if (corners) form.append('corners', JSON.stringify(corners));
  try {
    const r = await api.upload('/api/session/photo', form);
    show(r.session);
  } catch (e) {
    if (e.code === 'processing_failed' && e.details?.staged_file) return cornerEditor(e.details.staged_file);
    showError(e);
  }
}

function cornerEditor(staged) {
  const img = h('img', { src: `/staging/${staged}`, class: 'photo' });
  const box = h('div', { class: 'overlay-box' }, img);
  const editor = h('div', { class: 'card' }, h('p', {}, 'The paper wasn\'t found. Drag the four corners onto the paper corners.'), box);
  let overlay = null;
  img.addEventListener('load', () => {
    const w = img.naturalWidth;
    const hh = img.naturalHeight;
    overlay = createOverlay(box, img, { top_left: { x: 0, y: 0 }, top_right: { x: w, y: 0 }, bottom_right: { x: w, y: hh }, bottom_left: { x: 0, y: hh } }, () => {});
  });
  editor.append(h('button', { class: 'primary', onclick: () => overlay && act('POST', '/api/session/photo', { staged_file: staged, corners: overlay.handles }) }, 'Detect shots'));
  root.replaceChildren(editor);
}

function photoUpload(label) {
  const input = h('input', { type: 'file', accept: 'image/jpeg,image/png', onchange: () => input.files[0] && uploadPhoto(input.files[0]) });
  return h('label', { class: 'field upload' }, label, input);
}

function setup() {
  const mode = session.setup.input_mode;
  const parts = [h('h1', {}, `New ${mode} session`), setupForm(), notes()];
  if (mode === 'camera') parts.push(liveView());
  const ready = session.setup.device_id && session.setup.target_type_id;
  if (mode === 'photo') parts.push(ready ? photoUpload('Photo of the shot target') : h('p', { class: 'muted' }, 'Choose device and target type first.'));
  else parts.push(h('button', { class: 'primary', disabled: !ready, onclick: () => act('POST', '/api/session/start') }, 'Start'));
  parts.push(h('button', { onclick: () => act('POST', '/api/session/cancel') }, 'Cancel'));
  return h('div', { class: 'card' }, parts);
}

function notes() {
  return h('label', { class: 'field' }, 'Notes', h('textarea', { maxlength: 10000, onchange: (e) => act('PUT', '/api/session/notes', { notes: e.target.value || null }) }, session.notes || ''));
}

function scores() {
  const a = session.analysis;
  const updating = session.analysis_outdated ? h('span', { class: 'muted' }, ' updating…') : null;
  return h('div', { class: 'scores' },
    h('div', { class: 'score' }, h('span', {}, 'Score'), h('strong', {}, a ? a.score : '—')),
    h('div', { class: 'score' }, h('span', {}, 'Grouping score'), h('strong', {}, a ? a.grouping_score : '—')),
    h('div', { class: 'score' }, h('span', {}, 'Shots'), h('strong', {}, session.shots.length)),
    h('div', { class: 'score' }, h('span', {}, 'Time'), h('strong', { id: 'running-time' }, fmtDuration(session.duration_s))),
    updating);
}

function shotList() {
  const labels = shotLabels(session.shots, session.paper_no);
  const scoresList = session.analysis?.shot_scores || [];
  const list = h('ol', { class: 'shot-list' });
  session.shots.forEach((s, i) => {
    const low = s.confidence != null && s.confidence < 0.5;
    list.append(h('li', { class: `${i === selected ? 'selected' : ''} ${low ? 'low-confidence' : ''}`, onclick: () => { selected = i; show(session); } },
      h('span', {}, labels[i]), h('span', {}, scoresList[i] ?? ''), h('span', { class: 'muted' }, s.origin),
      h('button', { class: 'small', onclick: (e) => { e.stopPropagation(); act('DELETE', `/api/session/shots/${i}`); } }, '✕')));
  });
  return list;
}

function svgTarget(editable) {
  const t = target();
  if (!t) return h('p', { class: 'muted' }, 'No target type.');
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  const paperImage = session.setup.input_mode === 'photo' ? session.paper_images.find((p) => p.paper_no === (reviewPaper ?? session.paper_no)) : null;
  const shots = session.shots.map((s) => ({ ...s }));
  renderTarget(svg, t, shots, {
    bulletDiameterMm: device()?.bullet_diameter_mm ?? 4.5,
    paperCount: session.paper_no,
    image: paperImage ? `/staging/${paperImage.staged_file}` : null,
    selected,
    shift: session.state === 'review' ? session.analysis?.grouping_shift : null,
    mean: session.state === 'review' ? session.analysis?.mean_point : null,
  });
  if (!editable) return svg;
  let dragging = null;
  svg.addEventListener('pointerdown', (e) => {
    const idx = e.target.dataset?.index;
    if (idx !== undefined && session.state === 'review') {
      dragging = Number(idx);
      svg.setPointerCapture(e.pointerId);
    }
  });
  svg.addEventListener('pointermove', (e) => {
    if (dragging === null) return;
    const p = svgPoint(svg, e);
    const c = svg.querySelector(`circle[data-index="${dragging}"]`);
    c?.setAttribute('cx', p.x);
    c?.setAttribute('cy', p.y);
  });
  svg.addEventListener('pointerup', (e) => {
    const p = svgPoint(svg, e);
    if (dragging !== null) {
      const index = dragging;
      dragging = null;
      const shot = paperToShot(t, p.x, p.y);
      const before = shotToPaper(t, session.shots[index]);
      if (Math.hypot(before.x - p.x, before.y - p.y) > 0.2) act('PUT', `/api/session/shots/${index}`, shot);
      return;
    }
    if (e.target.dataset?.index !== undefined) return;
    const body = paperToShot(t, p.x, p.y);
    if (session.state === 'review' && reviewPaper) body.paper_no = reviewPaper;
    act('POST', '/api/session/shots', body);
  });
  return svg;
}

// doc/programs/web_server.md#session-view-per-state
function live() {
  const mode = session.setup.input_mode;
  const running = session.state === 'running';
  const main = mode === 'camera' ? liveView() : svgTarget(mode === 'manual' && running);
  return h('div', { class: 'layout' },
    h('div', { class: 'main' }, main),
    h('aside', {},
      scores(),
      h('div', { class: 'buttons' },
        running ? h('button', { onclick: () => act('POST', '/api/session/pause') }, 'Pause') : h('button', { class: 'primary', onclick: () => act('POST', '/api/session/resume') }, 'Resume'),
        h('button', { onclick: () => act('POST', '/api/session/new-paper') }, 'New paper'),
        h('button', { class: 'primary', onclick: () => act('POST', '/api/session/stop') }, 'Stop')),
      h('p', { class: 'muted' }, `Paper ${session.paper_no}`),
      shotList(),
      notes()));
}

function review() {
  const a = session.analysis;
  const papers = Array.from({ length: session.paper_no }, (_, i) => i + 1);
  const paperSelect = session.paper_no > 1 ? h('label', { class: 'field' }, 'Paper', select(papers.map((p) => ({ value: p, label: `Paper ${p}` })), reviewPaper ?? session.paper_no, (v) => { reviewPaper = Number(v); show(session); })) : null;
  return h('div', { class: 'layout' },
    h('div', { class: 'main' }, svgTarget(true), paperSelect, session.setup.input_mode === 'photo' ? photoUpload('Add photo (another paper)') : null),
    h('aside', {},
      scores(),
      a ? h('p', { class: 'muted' }, `Grouping shift: ${a.grouping_shift.x_mm.toFixed(1)} mm right, ${a.grouping_shift.y_mm.toFixed(1)} mm up`) : null,
      shotList(),
      notes(),
      h('p', { class: 'note' }, 'After saving, the session can\'t be changed.'),
      h('div', { class: 'buttons' },
        h('button', { class: 'primary', onclick: save }, 'Save'),
        h('button', { onclick: cancel }, 'Cancel'))));
}

async function save() {
  const r = await act('POST', '/api/session/save');
  if (r?.warnings?.length) showError({ message: r.warnings.join('; ') });
  reviewPaper = null;
  selected = null;
}

async function cancel() {
  if (await confirmDialog('Discard session', 'The shots of this session will be lost.')) act('POST', '/api/session/cancel');
}

function recovered() {
  const can = session.shots.length > 0 && session.setup.device_id && session.setup.target_type_id;
  return h('div', { class: 'card' },
    h('h1', {}, 'Unsaved session found'),
    h('p', {}, `An unsaved session from ${fmtTime(session.started_at)} with ${session.shots.length} shots was found.`),
    can ? null : h('p', { class: 'muted' }, 'It can\'t be saved: it needs at least one shot, a device and a target type.'),
    h('div', { class: 'buttons' },
      h('button', { class: 'primary', disabled: !can, onclick: save }, 'Save'),
      h('button', { onclick: () => act('POST', '/api/session/cancel') }, 'Discard')));
}
