// doc/programs/web_server.md#target-type-editor

import * as api from '../api.js';
import { createOverlay } from '../overlay.js';
import { generateRings, renderTarget, svgPoint, validateTargetType } from '../target.js';
import { confirmDialog, h, run, showError } from '../ui.js';

const SVG = 'http://www.w3.org/2000/svg';
const FIELDS = [
  ['center_x_mm', 'Center x (mm)'], ['center_y_mm', 'Center y (mm)'],
  ['inner_white_radius_mm', 'Inner white radius (mm)'], ['inner_white_score', 'Inner white score'],
  ['black_radius_mm', 'Black radius (mm)'], ['black_score', 'Black score'],
  ['outer_radius_mm', 'Outermost radius (mm)'], ['outer_score', 'Outermost score'],
];

export async function render(el) {
  const { target_types } = await api.get('/api/target-types');
  el.replaceChildren(h('h1', {}, 'Target types'),
    h('button', { class: 'primary', onclick: () => pictureStep(el) }, 'New target type (from a picture)'),
    h('div', { class: 'grid' }, target_types.map((t) => card(el, t))));
}

function card(el, t) {
  const svg = document.createElementNS(SVG, 'svg');
  renderTarget(svg, t, []);
  return h('div', { class: 'card' }, h('h2', {}, t.name, t.retired ? ' (retired)' : ''), svg,
    h('p', { class: 'muted' }, `${t.paper_width_mm} × ${t.paper_height_mm} mm, rings ${t.inner_white_score}–${t.outer_score}${t.locked ? ', locked (used by a saved session)' : ''}`),
    h('button', { onclick: () => ringStep(el, { ...t }, null, t.locked) }, t.locked ? 'View' : 'Edit'),
    h('label', { class: 'check' }, h('input', { type: 'checkbox', checked: t.retired, onchange: (e) => run(() => api.put(`/api/target-types/${t.id}`, { ...t, retired: e.target.checked })) }), ' retired'),
    h('button', { class: 'danger', onclick: async () => (await confirmDialog('Delete target type', `Delete ${t.name}?`)) && (await run(() => api.del(`/api/target-types/${t.id}`))) && render(el) }, 'Delete'));
}

// doc/programs/web_server.md#target-type-editor
function pictureStep(el) {
  const file = h('input', { type: 'file', accept: 'image/jpeg,image/png', onchange: async () => {
    const f = new FormData();
    f.append('photo', file.files[0]);
    await next(() => api.upload('/api/target-types/photo', f));
  } });
  const next = async (call) => {
    try {
      paperStep(el, await call());
    } catch (e) {
      if (e.details?.staged_file) paperStep(el, { staged_file: e.details.staged_file, corners: null });
      else showError(e);
    }
  };
  el.replaceChildren(h('h1', {}, 'New target type: picture'),
    h('div', { class: 'card' },
      h('button', { class: 'primary', onclick: () => next(() => api.post('/api/target-types/photo/capture')) }, 'Take a picture with the camera'),
      h('label', { class: 'field' }, 'or upload a photo', file),
      h('button', { onclick: () => render(el) }, 'Cancel')));
}

function paperStep(el, found) {
  const img = h('img', { src: `/staging/${found.staged_file}`, class: 'photo' });
  const box = h('div', { class: 'overlay-box' }, img);
  const width = h('input', { type: 'number', min: 1, step: '0.1', value: 170 });
  const height = h('input', { type: 'number', min: 1, step: '0.1', value: 170 });
  let overlay = null;
  img.addEventListener('load', () => {
    const w = img.naturalWidth;
    const hh = img.naturalHeight;
    const c = found.corners || { top_left: { x: 0, y: 0 }, top_right: { x: w, y: 0 }, bottom_right: { x: w, y: hh }, bottom_left: { x: 0, y: hh } };
    overlay = createOverlay(box, img, structuredClone(c), () => {});
  });
  const prepare = async () => {
    const r = await run(() => api.post('/api/target-types/photo/prepare', {
      staged_file: found.staged_file, corners: overlay.handles, paper_width_mm: Number(width.value), paper_height_mm: Number(height.value),
    }));
    if (!r) return;
    const W = Number(width.value);
    const H = Number(height.value);
    const p = r.proposal;
    const t = {
      name: '', retired: false, paper_width_mm: W, paper_height_mm: H,
      center_x_mm: p.center_x_mm ?? W / 2, center_y_mm: p.center_y_mm ?? H / 2,
      inner_white_radius_mm: p.inner_white_radius_mm ?? Math.min(W, H) / 40, inner_white_score: 12,
      black_radius_mm: p.black_radius_mm ?? Math.min(W, H) / 6, black_score: null,
      outer_radius_mm: p.outer_radius_mm ?? Math.min(W, H) / 2.4, outer_score: 1,
    };
    ringStep(el, t, `/staging/${r.staged_file}`, false);
  };
  el.replaceChildren(h('h1', {}, 'New target type: paper'),
    h('p', {}, found.corners ? 'Check the paper corners and correct them if needed.' : 'No paper was found: drag the handles onto the paper corners.'),
    box,
    h('div', { class: 'row' }, h('label', { class: 'field' }, 'Paper width (mm)', width), h('label', { class: 'field' }, 'Paper height (mm)', height)),
    h('div', { class: 'buttons' }, h('button', { class: 'primary', onclick: prepare }, 'Next'), h('button', { onclick: () => render(el) }, 'Cancel')));
}

// doc/programs/web_server.md#target-type-editor
function ringStep(el, t, image, readOnly) {
  const svg = document.createElementNS(SVG, 'svg');
  const inputs = {};
  const error = h('p', { class: 'error' });
  const draw = () => {
    const valid = typeof t.black_score === 'number' ? validateTargetType(t) : 'black_score is required';
    error.textContent = valid || '';
    const shown = { ...t, black_score: t.black_score ?? t.inner_white_score - 1 };
    renderTarget(svg, shown, [], { image });
    if (!readOnly) {
      const handle = (cx, cy, key) => {
        const c = document.createElementNS(SVG, 'circle');
        c.setAttribute('cx', cx);
        c.setAttribute('cy', cy);
        c.setAttribute('r', Math.max(1.5, t.paper_width_mm / 120));
        c.setAttribute('class', 'handle-svg');
        c.dataset.handle = key;
        svg.append(c);
      };
      handle(t.center_x_mm, t.center_y_mm, 'center');
      for (const k of ['inner_white_radius_mm', 'black_radius_mm', 'outer_radius_mm']) handle(t.center_x_mm + t[k], t.center_y_mm, k);
    }
    for (const [k] of FIELDS) if (document.activeElement !== inputs[k]) inputs[k].value = t[k] ?? '';
    rings.textContent = typeof t.black_score === 'number' && !validateTargetType(t) ? generateRings(t).map((r) => `${r.score}: ${r.radius_mm.toFixed(2)}`).join(' · ') : '';
  };
  const rings = h('p', { class: 'muted' });
  let drag = null;
  svg.addEventListener('pointerdown', (e) => {
    if (e.target.dataset?.handle) {
      drag = e.target.dataset.handle;
      svg.setPointerCapture(e.pointerId);
    }
  });
  svg.addEventListener('pointermove', (e) => {
    if (!drag) return;
    const p = svgPoint(svg, e);
    if (drag === 'center') {
      t.center_x_mm = round(p.x);
      t.center_y_mm = round(p.y);
    } else t[drag] = round(Math.max(0.1, Math.hypot(p.x - t.center_x_mm, p.y - t.center_y_mm)));
    draw();
  });
  svg.addEventListener('pointerup', () => (drag = null));
  const fields = FIELDS.map(([k, label]) => h('label', { class: 'field' }, label,
    (inputs[k] = h('input', { type: 'number', step: k.endsWith('score') ? 1 : 0.05, disabled: readOnly, oninput: (e) => { t[k] = e.target.value === '' ? null : Number(e.target.value); draw(); } }))));
  const name = h('input', { value: t.name || '', disabled: readOnly && !!t.id });
  const save = async () => {
    const body = { ...t, name: name.value.trim() };
    delete body.rings;
    delete body.locked;
    const r = body.id ? await run(() => api.put(`/api/target-types/${body.id}`, body)) : await run(() => api.post('/api/target-types', body));
    if (r) render(el);
  };
  const copy = () => {
    const c = { ...t, name: '' };
    delete c.id;
    delete c.uuid;
    ringStep(el, c, null, false);
  };
  draw();
  el.replaceChildren(h('h1', {}, readOnly ? `${t.name} (locked)` : 'Target type: rings'),
    h('div', { class: 'layout' },
      h('div', { class: 'main' }, svg),
      h('aside', {}, h('div', { class: 'form' }, fields), rings, error,
        h('label', { class: 'field' }, 'Name', name),
        h('div', { class: 'buttons' },
          readOnly ? h('button', { class: 'primary', onclick: copy }, 'Copy and edit') : h('button', { class: 'primary', onclick: save }, 'Save'),
          h('button', { onclick: () => render(el) }, readOnly ? 'Back' : 'Cancel')))));
}

function round(v) {
  return Math.round(v * 20) / 20;
}
