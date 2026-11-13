// doc/programs/web_server.md#frontend

import * as api from '../api.js';
import { on } from '../events.js';
import { fmtTime, h, run } from '../ui.js';

let off = null;

export async function render(el) {
  const { programs } = await api.get('/api/system');
  el.replaceChildren(h('h1', {}, 'System'),
    h('p', {}, `Open the web UI on other devices: ${location.origin}`),
    h('table', { class: 'table' },
      h('thead', {}, h('tr', {}, ['Program', 'State', 'PID', 'Restarts', 'Since', ''].map((t) => h('th', {}, t)))),
      h('tbody', {}, programs.map((p) => h('tr', { class: `state-${p.state}` },
        h('td', {}, p.name), h('td', {}, p.state), h('td', {}, p.pid ?? '—'), h('td', {}, p.restarts), h('td', {}, fmtTime(Math.floor(p.since_ms / 1000))),
        h('td', {}, h('button', { onclick: () => run(() => api.post(`/api/system/programs/${p.name}/restart`)) }, 'Restart')))))));
  off?.();
  off = on('program_state_changed', () => render(el));
}

export function leave() {
  off?.();
  off = null;
}
