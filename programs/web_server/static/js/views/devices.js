// doc/programs/web_server.md#frontend

import * as api from '../api.js';
import { confirmDialog, h, run } from '../ui.js';

const FIELDS = [['name', 'Name'], ['category', 'Category'], ['caliber', 'Caliber'], ['description', 'Description']];

function form(d, onSave) {
  const inputs = {};
  const rows = FIELDS.map(([k, label]) => h('label', { class: 'field' }, label, (inputs[k] = h(k === 'description' ? 'textarea' : 'input', { value: d[k] ?? '' }, k === 'description' ? d[k] ?? '' : null))));
  const bullet = h('input', { type: 'number', step: '0.01', min: '0.01', value: d.bullet_diameter_mm ?? 4.5, disabled: d.locked });
  return h('div', { class: 'form' }, rows,
    h('label', { class: 'field' }, 'Bullet diameter (mm)', bullet, d.locked ? h('span', { class: 'muted' }, ' locked: used by a saved session') : null),
    h('button', { class: 'primary', onclick: () => {
      const body = { ...d };
      for (const [k] of FIELDS) body[k] = inputs[k].value.trim() || (k === 'name' ? '' : null);
      body.bullet_diameter_mm = Number(bullet.value);
      onSave(body);
    } }, 'Save'));
}

export async function render(el) {
  const { devices } = await api.get('/api/devices');
  const card = (d) => {
    const file = h('input', { type: 'file', accept: 'image/jpeg,image/png', onchange: async () => {
      const f = new FormData();
      f.append('image', file.files[0]);
      (await run(() => api.upload(`/api/devices/${d.id}/image`, f, 'PUT'))) && render(el);
    } });
    return h('div', { class: 'card' },
      h('h2', {}, d.name, d.retired ? ' (retired)' : ''),
      d.image_path ? h('img', { class: 'device-image', src: `/images/${d.image_path}`, alt: d.name }) : null,
      h('p', { class: 'muted' }, `Shots fired: ${d.shot_count}`),
      form(d, async (body) => (await run(() => api.put(`/api/devices/${d.id}`, body))) && render(el)),
      h('label', { class: 'field' }, 'Image', file),
      d.image_path ? h('button', { onclick: async () => (await run(() => api.del(`/api/devices/${d.id}/image`))) && render(el) }, 'Remove image') : null,
      h('label', { class: 'check' }, h('input', { type: 'checkbox', checked: d.retired, onchange: (e) => run(() => api.put(`/api/devices/${d.id}`, { ...d, retired: e.target.checked })) }), ' retired'),
      h('button', { class: 'danger', onclick: async () => (await confirmDialog('Delete device', `Delete ${d.name}?`)) && (await run(() => api.del(`/api/devices/${d.id}`))) && render(el) }, 'Delete'));
  };
  el.replaceChildren(h('h1', {}, 'Devices'),
    h('details', { class: 'card' }, h('summary', {}, 'New device'), form({ retired: false }, async (body) => (await run(() => api.post('/api/devices', body))) && render(el))),
    devices.map(card));
}
