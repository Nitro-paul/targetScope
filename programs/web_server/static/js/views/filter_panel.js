// doc/programs/web_server.md#statistics-page
// doc/interfaces.md#sessionfilter

import * as api from '../api.js';

const LISTS = ['session_ids', 'shooter_ids', 'device_ids', 'device_categories', 'calibers', 'target_type_ids'];
const NUMBER_LISTS = new Set(['session_ids', 'shooter_ids', 'device_ids', 'target_type_ids']);
const RANGES = { distance_m: ['min', 'max'], saved_at: ['from', 'to'], shot_count: ['min', 'max'] };
const BOOLS = ['single_handed', 'supported', 'include_retired'];

// doc/programs/web_server.md#statistics-page
export function filterToQuery(filter) {
  const q = new URLSearchParams();
  for (const k of LISTS) if (Array.isArray(filter[k])) q.set(k, JSON.stringify(filter[k]));
  for (const [k, [a, b]] of Object.entries(RANGES)) {
    const r = filter[k];
    if (r && typeof r === 'object') {
      if (r[a] != null) q.set(`${k}.${a}`, String(r[a]));
      if (r[b] != null) q.set(`${k}.${b}`, String(r[b]));
      if (r[a] == null && r[b] == null) q.set(k, '{}');
    }
  }
  for (const k of BOOLS) if (typeof filter[k] === 'boolean') q.set(k, String(filter[k]));
  return q.toString();
}

export function queryToFilter(query) {
  const q = new URLSearchParams(query);
  const f = {};
  for (const k of LISTS) {
    if (!q.has(k)) continue;
    try {
      const list = JSON.parse(q.get(k));
      if (Array.isArray(list)) f[k] = list.map((v) => (NUMBER_LISTS.has(k) ? Number(v) : String(v)));
    } catch {}
  }
  for (const [k, [a, b]] of Object.entries(RANGES)) {
    const r = {};
    if (q.has(`${k}.${a}`)) r[a] = Number(q.get(`${k}.${a}`));
    if (q.has(`${k}.${b}`)) r[b] = Number(q.get(`${k}.${b}`));
    if (Object.keys(r).length || q.has(k)) f[k] = r;
  }
  for (const k of BOOLS) if (q.has(k)) f[k] = q.get(k) === 'true';
  return f;
}

function multiSelect(label, items, selected, onChange) {
  const box = document.createElement('label');
  box.className = 'field';
  box.textContent = label;
  const sel = document.createElement('select');
  sel.multiple = true;
  for (const it of items) {
    const o = document.createElement('option');
    o.value = it.value;
    o.textContent = it.label;
    o.selected = selected?.includes(it.value);
    sel.appendChild(o);
  }
  sel.addEventListener('change', () => onChange([...sel.selectedOptions].map((o) => o.value)));
  box.appendChild(sel);
  return box;
}

function triState(label, value, onChange) {
  const box = document.createElement('label');
  box.className = 'field';
  box.textContent = label;
  const sel = document.createElement('select');
  for (const [v, text] of [['', 'any'], ['true', 'yes'], ['false', 'no']]) {
    const o = document.createElement('option');
    o.value = v;
    o.textContent = text;
    o.selected = String(value ?? '') === v;
    sel.appendChild(o);
  }
  sel.addEventListener('change', () => onChange(sel.value === '' ? undefined : sel.value === 'true'));
  box.appendChild(sel);
  return box;
}

function range(label, value, keys, onChange, kind = 'number') {
  const box = document.createElement('div');
  box.className = 'field range';
  box.append(label);
  for (const key of keys) {
    const input = document.createElement('input');
    input.type = kind;
    input.placeholder = key;
    const v = value?.[key];
    if (v != null) input.value = kind === 'date' ? new Date(v * 1000).toISOString().slice(0, 10) : v;
    input.addEventListener('change', () => {
      const next = { ...(value || {}) };
      if (input.value === '') delete next[key];
      else next[key] = kind === 'date' ? Math.floor(new Date(input.value).getTime() / 1000) + (key === 'to' ? 86399 : 0) : Number(input.value);
      onChange(Object.keys(next).length ? next : undefined);
    });
    box.appendChild(input);
  }
  return box;
}

// doc/programs/web_server.md#statistics-page
export async function renderFilterPanel(el, filter, onChange) {
  const [shooters, devices, targets] = await Promise.all([api.get('/api/shooters'), api.get('/api/devices'), api.get('/api/target-types')]);
  const set = (k, v) => {
    const next = { ...filter };
    if (v === undefined || (Array.isArray(v) && v.length === 0 && !LISTS.includes(k))) delete next[k];
    else next[k] = v;
    filter = next;
    onChange(next);
  };
  const ids = (list) => list.map((x) => ({ value: x.id, label: x.name + (x.retired ? ' (retired)' : '') }));
  const unique = (values) => [...new Set(values.filter(Boolean))].sort().map((v) => ({ value: v, label: v }));
  const panel = document.createElement('div');
  panel.className = 'filter-panel';
  const multi = (label, key, items, numeric) =>
    multiSelect(label, items, filter[key]?.map(String), (vals) => set(key, vals.length ? vals.map((v) => (numeric ? Number(v) : v)) : undefined));
  panel.append(
    multi('Shooters', 'shooter_ids', ids(shooters.shooters).map((i) => ({ ...i, value: String(i.value) })), true),
    multi('Devices', 'device_ids', ids(devices.devices).map((i) => ({ ...i, value: String(i.value) })), true),
    multi('Device categories', 'device_categories', unique(devices.devices.map((d) => d.category)), false),
    multi('Calibers', 'calibers', unique(devices.devices.map((d) => d.caliber)), false),
    multi('Target types', 'target_type_ids', ids(targets.target_types).map((i) => ({ ...i, value: String(i.value) })), true),
    range('Distance (m)', filter.distance_m, ['min', 'max'], (v) => set('distance_m', v)),
    range('Saved', filter.saved_at, ['from', 'to'], (v) => set('saved_at', v), 'date'),
    range('Shots', filter.shot_count, ['min', 'max'], (v) => set('shot_count', v)),
    triState('Single-handed', filter.single_handed, (v) => set('single_handed', v)),
    triState('Supported', filter.supported, (v) => set('supported', v)),
  );
  const shortcuts = document.createElement('div');
  shortcuts.className = 'shortcuts';
  for (const [label, days] of [['Last 7 days', 7], ['Last 30 days', 30], ['Last year', 365]]) {
    const b = document.createElement('button');
    b.textContent = label;
    b.type = 'button';
    b.addEventListener('click', () => set('saved_at', { from: Math.floor(Date.now() / 1000) - days * 86400 }));
    shortcuts.appendChild(b);
  }
  const retired = document.createElement('label');
  retired.className = 'field check';
  const cb = document.createElement('input');
  cb.type = 'checkbox';
  cb.checked = filter.include_retired !== false;
  cb.addEventListener('change', () => set('include_retired', cb.checked ? undefined : false));
  retired.append(cb, ' include retired');
  panel.append(shortcuts, retired);
  el.replaceChildren(panel);
}

export async function render() {}
