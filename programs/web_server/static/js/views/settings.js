// doc/programs/web_server.md#settings-view

import * as api from '../api.js';
import { confirmDialog, dialog, h, run, showInfo } from '../ui.js';
import { checkValueRule } from './settings_rules.js';

const HIDDEN = new Set(['camera_controls', 'lens_calibrations', 'admin_password_hash']);

export { checkValueRule };

// doc/programs/web_server.md#settings-view
async function changeImageDir(path) {
  const move = await dialog('Image folder', h('p', {}, 'Move the existing images to the new folder?'), [['Cancel', null], ['Don\'t move', false], ['Move images', true, 'primary']]);
  if (move === null) return null;
  return { path, move_images: move };
}

function report(r) {
  if (r?.images) showInfo(`${r.images.moved} images moved${r.images.failed.length ? `; not moved: ${r.images.failed.join(', ')}` : ''}`);
}

export async function render(el) {
  const { settings } = await api.get('/api/settings');
  const admin = await api.get('/api/admin');
  const row = (s) => {
    const input = h('input', { value: s.value, class: s.active ? '' : 'inactive' });
    const err = h('span', { class: 'error' });
    const active = s.optional ? h('input', { type: 'checkbox', checked: s.active, onchange: () => input.classList.toggle('inactive', !active.checked) }) : null;
    input.addEventListener('input', () => (err.textContent = checkValueRule(s.value_rule, input.value) || ''));
    const save = async () => {
      const problem = checkValueRule(s.value_rule, input.value);
      if (problem && (!active || active.checked)) return (err.textContent = problem);
      if (s.name === 'image_dir') {
        const body = await changeImageDir(input.value);
        if (body) report(await run(() => api.put('/api/settings/image_dir', body)));
      } else await run(() => api.put(`/api/settings/${s.name}`, { value: input.value, active: active ? active.checked : true }));
      render(el);
    };
    const reset = async () => {
      let move = false;
      if (s.name === 'image_dir') {
        const body = await changeImageDir(s.default_value);
        if (!body) return;
        move = body.move_images;
      }
      report(await run(() => api.post('/api/settings/reset', { names: [s.name], move_images: move })));
      render(el);
    };
    return h('tr', {},
      h('td', {}, s.display_name, h('div', { class: 'muted' }, s.value_rule)),
      h('td', {}, active, input, err),
      h('td', {}, h('button', { onclick: save }, 'Save'), h('button', { onclick: reset }, 'Restore default')));
  };
  const resetAll = async () => {
    if (!(await confirmDialog('Restore all defaults', 'All settings are restored to their defaults. The admin password and the lens calibrations are kept.'))) return;
    const body = await changeImageDir('');
    if (!body) return;
    report(await run(() => api.post('/api/settings/reset', { names: null, move_images: body.move_images })));
    render(el);
  };
  const pw = h('input', { type: 'password', autocomplete: 'new-password' });
  const current = h('input', { type: 'password', autocomplete: 'current-password' });
  const setPw = async (remove) => {
    const r = await run(() => api.put('/api/admin/password', { current_password: current.value || null, new_password: remove ? null : pw.value }));
    if (r) {
      showInfo(r.protected ? 'Admin password set' : 'Admin password removed');
      render(el);
    }
  };
  el.replaceChildren(h('h1', {}, 'Settings'),
    h('table', { class: 'table' }, h('tbody', {}, settings.filter((s) => !HIDDEN.has(s.name)).map(row))),
    h('button', { onclick: resetAll }, 'Restore all defaults'),
    h('section', { class: 'card' }, h('h2', {}, 'Admin password'),
      h('p', {}, admin.protected ? 'Destructive actions need the admin password.' : 'No admin password: everything is open.'),
      admin.protected ? h('label', { class: 'field' }, 'Current password', current) : null,
      h('label', { class: 'field' }, 'New password', pw),
      h('div', { class: 'buttons' }, h('button', { class: 'primary', onclick: () => setPw(false) }, 'Set password'),
        admin.protected ? h('button', { onclick: () => setPw(true) }, 'Remove password') : null,
        admin.logged_in ? h('button', { onclick: async () => { await api.post('/api/admin/logout'); render(el); } }, 'Log out') : null)));
}
