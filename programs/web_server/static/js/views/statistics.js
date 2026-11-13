// doc/programs/web_server.md#statistics-page

import * as api from '../api.js';
import { renderTarget } from '../target.js';
import { h, run, select } from '../ui.js';
import { filterToQuery, queryToFilter, renderFilterPanel } from './filter_panel.js';

const SVG = 'http://www.w3.org/2000/svg';
const GROUP_BY = ['none', 'shooter', 'device', 'device_category', 'distance', 'single_handed', 'supported'];
const PERIODS = ['session', 'day', 'week', 'month'];

export async function render(el, params) {
  const filter = queryToFilter(params.toString());
  const groupBy = params.get('group_by') || 'none';
  const period = params.get('period') || 'session';
  const go = (f, g = groupBy, p = period) => {
    const q = new URLSearchParams(filterToQuery(f));
    q.set('group_by', g);
    q.set('period', p);
    location.hash = `#/statistics?${q}`;
  };
  const panel = h('div');
  const results = h('div', {}, h('p', { class: 'muted' }, 'Calculating…'));
  el.replaceChildren(h('h1', {}, 'Statistics'), panel,
    h('div', { class: 'row' },
      h('label', { class: 'field' }, 'Group by', select(GROUP_BY.map((g) => ({ value: g, label: g.replace('_', ' ') })), groupBy, (v) => go(filter, v, period))),
      h('label', { class: 'field' }, 'Period', select(PERIODS.map((p) => ({ value: p, label: p })), period, (v) => go(filter, groupBy, v)))),
    results);
  await renderFilterPanel(panel, filter, (f) => go(f));
  const r = await run(() => api.post('/api/statistics', { filter, group_by: groupBy, period }));
  if (!r) return;
  const [targets, shooters, devices] = await Promise.all([api.get('/api/target-types'), api.get('/api/shooters'), api.get('/api/devices')]);
  const label = (key) => {
    if (!key) return 'all';
    const [k, v] = Object.entries(key)[0];
    if (v === null) return k === 'shooter_id' ? 'no shooter' : 'not set';
    if (k === 'shooter_id') return shooters.shooters.find((s) => s.id === v)?.name ?? v;
    if (k === 'device_id') return devices.devices.find((d) => d.id === v)?.name ?? v;
    return `${v}`;
  };
  if (!r.groups.length) return results.replaceChildren(h('p', {}, 'No sessions match the filter.'));
  const byTarget = new Map();
  for (const g of r.groups) byTarget.set(g.target_type_id, [...(byTarget.get(g.target_type_id) || []), g]);
  results.replaceChildren(h('p', {}, `${r.session_count} sessions`), ...[...byTarget].map(([id, groups]) => section(targets.target_types.find((t) => t.id === id), groups, label)));
}

function section(t, groups, label) {
  const fmt = (v, d = 1) => (typeof v === 'number' ? v.toFixed(d) : v);
  const table = h('table', { class: 'table' },
    h('thead', {}, h('tr', {}, ['Group', 'Sessions', 'Shots', 'Score avg', 'min', 'max', 'Score/shot', 'Grouping/shot', 'Spread (mm)', 'Mean radius (mm)', 'Mean point (mm)'].map((x) => h('th', {}, x)))),
    h('tbody', {}, groups.map((g) => {
      const s = g.summary;
      return h('tr', {}, h('td', {}, label(g.group_key)), h('td', {}, s.session_count), h('td', {}, s.shot_count), h('td', {}, fmt(s.score_avg)), h('td', {}, s.score_min), h('td', {}, s.score_max),
        h('td', {}, fmt(s.score_per_shot_avg, 2)), h('td', {}, fmt(s.grouping_score_per_shot_avg, 2)), h('td', {}, fmt(s.extreme_spread_mm_avg)), h('td', {}, fmt(s.mean_radius_mm_avg)),
        h('td', {}, `${fmt(s.mean_point_avg.x_mm)}, ${fmt(s.mean_point_avg.y_mm)}`));
    })));
  const distribution = document.createElementNS(SVG, 'svg');
  let chosen = 0;
  const drawShots = () => t && renderTarget(distribution, t, groups[chosen].shots.map((s) => ({ ...s, paper_no: 1 })), { bulletDiameterMm: 1.5 });
  drawShots();
  return h('section', { class: 'card' }, h('h2', {}, t?.name ?? `Target type ${groups[0].target_type_id}`), table,
    h('h3', {}, 'Trend'), trend(groups, label),
    h('h3', {}, 'Shot distribution'),
    select(groups.map((g, i) => ({ value: i, label: label(g.group_key) })), 0, (v) => { chosen = Number(v); drawShots(); }),
    distribution);
}

// doc/programs/web_server.md#statistics-page
function trend(groups, label) {
  const W = 800, H = 260, P = 40;
  const all = groups.flatMap((g) => g.trend);
  if (!all.length) return h('p', {}, '—');
  const t0 = Math.min(...all.map((e) => e.period_start));
  const t1 = Math.max(...all.map((e) => e.period_start));
  const yMax = Math.max(...all.map((e) => Math.max(e.values.score_per_shot_avg, e.values.grouping_score_per_shot_avg)), 1);
  const sMax = Math.max(...all.map((e) => e.values.extreme_spread_mm_avg), 1);
  const x = (t) => P + ((t - t0) / Math.max(1, t1 - t0)) * (W - 2 * P);
  const y = (v, max) => H - P - (v / max) * (H - 2 * P);
  const svg = document.createElementNS(SVG, 'svg');
  svg.setAttribute('viewBox', `0 0 ${W} ${H}`);
  svg.setAttribute('class', 'chart');
  const add = (tag, attrs, text) => {
    const e = document.createElementNS(SVG, tag);
    for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
    if (text) e.textContent = text;
    svg.append(e);
  };
  add('line', { x1: P, y1: H - P, x2: W - P, y2: H - P, class: 'axis' });
  add('line', { x1: P, y1: P, x2: P, y2: H - P, class: 'axis' });
  add('text', { x: 4, y: P - 8, class: 'axis-label' }, `score/shot (max ${yMax.toFixed(1)})`);
  add('text', { x: W - P, y: P - 8, class: 'axis-label', 'text-anchor': 'end' }, `spread mm (max ${sMax.toFixed(1)})`);
  groups.forEach((g, i) => {
    const line = (key, max, cls) => add('polyline', { points: g.trend.map((e) => `${x(e.period_start)},${y(e.values[key], max)}`).join(' '), class: `series s${i % 6} ${cls}` });
    line('score_per_shot_avg', yMax, 'score');
    line('grouping_score_per_shot_avg', yMax, 'grouping');
    line('extreme_spread_mm_avg', sMax, 'spread');
    add('text', { x: W - P + 4, y: P + 14 * i, class: `legend s${i % 6}` }, label(g.group_key));
  });
  return svg;
}
