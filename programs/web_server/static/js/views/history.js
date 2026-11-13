// doc/programs/web_server.md#frontend

import * as api from '../api.js';
import { renderTarget } from '../target.js';
import { confirmDialog, fmtDuration, fmtTime, h, run } from '../ui.js';
import { filterToQuery, queryToFilter, renderFilterPanel } from './filter_panel.js';

const PAGE = 50;

export async function render(el, params) {
  const filter = queryToFilter(params.toString());
  const offset = Number(params.get('offset') || 0);
  if (params.get('id')) return detail(el, Number(params.get('id')));
  const panel = h('div');
  const list = h('div');
  el.replaceChildren(h('h1', {}, 'History'), panel, list);
  await renderFilterPanel(panel, filter, (f) => (location.hash = `#/history?${filterToQuery(f)}`));
  const [r, shooters, devices, targets] = await Promise.all([
    api.post('/api/sessions/query', { filter, limit: PAGE, offset }),
    api.get('/api/shooters'),
    api.get('/api/devices'),
    api.get('/api/target-types'),
  ]);
  const name = (list, id) => list.find((x) => x.id === id)?.name ?? '—';
  const rows = r.sessions.map((s) =>
    h('tr', { onclick: () => (location.hash = `#/history?id=${s.id}`) },
      h('td', {}, fmtTime(s.saved_at)), h('td', {}, name(shooters.shooters, s.shooter_id)), h('td', {}, name(devices.devices, s.device_id)),
      h('td', {}, name(targets.target_types, s.target_type_id)), h('td', {}, s.distance_m ?? '—'), h('td', {}, s.shot_count),
      h('td', {}, s.score), h('td', {}, s.grouping_score)));
  const page = (o) => { const p = new URLSearchParams(filterToQuery(filter)); p.set('offset', o); location.hash = `#/history?${p}`; };
  list.replaceChildren(
    h('table', { class: 'table clickable' },
      h('thead', {}, h('tr', {}, ['Saved', 'Shooter', 'Device', 'Target', 'Distance', 'Shots', 'Score', 'Grouping'].map((t) => h('th', {}, t)))),
      h('tbody', {}, rows)),
    h('div', { class: 'pager' },
      h('button', { disabled: offset === 0, onclick: () => page(Math.max(0, offset - PAGE)) }, '‹ Newer'),
      h('span', {}, `${r.total ? offset + 1 : 0}–${Math.min(offset + PAGE, r.total)} of ${r.total}`),
      h('button', { disabled: offset + PAGE >= r.total, onclick: () => page(offset + PAGE) }, 'Older ›')));
}

async function detail(el, id) {
  const s = await api.get(`/api/sessions/${id}`);
  const [targets, devices] = await Promise.all([api.get('/api/target-types'), api.get('/api/devices')]);
  const t = targets.target_types.find((x) => x.id === s.target_type_id);
  const d = devices.devices.find((x) => x.id === s.device_id);
  const a = t && d ? await api.post('/api/analyze', { shots: s.shots.map(({ angle_turns, radius_mm }) => ({ angle_turns, radius_mm })), target_type: t, bullet_diameter_mm: d.bullet_diameter_mm }) : null;
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  if (t) renderTarget(svg, t, s.shots, { bulletDiameterMm: d?.bullet_diameter_mm, paperCount: s.paper_count, shift: a?.grouping_shift, mean: a?.mean_point });
  const del = async () => {
    if (!(await confirmDialog('Delete session', 'The session, its shots and its images are deleted.'))) return;
    if (await run(() => api.del(`/api/sessions/${id}`))) location.hash = '#/history';
  };
  el.replaceChildren(
    h('a', { href: '#/history' }, '‹ History'),
    h('h1', {}, `Session ${fmtTime(s.saved_at)}`),
    h('div', { class: 'layout' },
      h('div', { class: 'main' }, svg),
      h('aside', {},
        h('dl', { class: 'facts' },
          h('dt', {}, 'Score'), h('dd', {}, s.score), h('dt', {}, 'Grouping score'), h('dd', {}, s.grouping_score),
          h('dt', {}, 'Shots'), h('dd', {}, s.shot_count), h('dt', {}, 'Papers'), h('dd', {}, s.paper_count),
          h('dt', {}, 'Time'), h('dd', {}, fmtDuration(s.duration_s)), h('dt', {}, 'Notes'), h('dd', {}, s.notes || '—')),
        a ? h('p', {}, `Shot scores: ${a.shot_scores.join(', ')}`) : null,
        h('div', { class: 'images' }, s.images.map((i) => h('figure', {}, h('img', { src: `/images/${i.image_path}`, alt: `Paper ${i.paper_no}` }), h('figcaption', {}, `Paper ${i.paper_no}`)))),
        h('button', { class: 'danger', onclick: del }, 'Delete session'))));
}
