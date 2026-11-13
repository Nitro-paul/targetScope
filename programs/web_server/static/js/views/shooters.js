// doc/programs/web_server.md#frontend

import * as api from '../api.js';
import { confirmDialog, h, run } from '../ui.js';

export async function render(el) {
  const { shooters } = await api.get('/api/shooters');
  const name = h('input', { placeholder: 'Name' });
  const add = async () => (await run(() => api.post('/api/shooters', { name: name.value, retired: false }))) && render(el);
  const row = (s) => {
    const input = h('input', { value: s.name });
    return h('tr', {},
      h('td', {}, input),
      h('td', {}, h('label', { class: 'check' }, h('input', { type: 'checkbox', checked: s.retired, onchange: (e) => run(() => api.put(`/api/shooters/${s.id}`, { ...s, retired: e.target.checked })) }), ' retired')),
      h('td', {}, h('button', { onclick: async () => (await run(() => api.put(`/api/shooters/${s.id}`, { ...s, name: input.value }))) && render(el) }, 'Rename'),
        h('button', { class: 'danger', onclick: async () => (await confirmDialog('Delete shooter', `Delete ${s.name}?`)) && (await run(() => api.del(`/api/shooters/${s.id}`))) && render(el) }, 'Delete')));
  };
  el.replaceChildren(h('h1', {}, 'Shooters'),
    h('div', { class: 'row' }, name, h('button', { class: 'primary', onclick: add }, 'Add')),
    h('table', { class: 'table' }, h('tbody', {}, shooters.map(row))));
}
