// doc/programs/web_server.md#frontend

import { connectEvents } from './events.js';
import { onAuthRequired, onSettingNotSet, post, put, get } from './api.js';
import { dialog, h, showError } from './ui.js';

const routes = {
  '/session': () => import('./views/session.js'),
  '/history': () => import('./views/history.js'),
  '/statistics': () => import('./views/statistics.js'),
  '/shooters': () => import('./views/shooters.js'),
  '/devices': () => import('./views/devices.js'),
  '/targets': () => import('./views/targets.js'),
  '/camera': () => import('./views/camera.js'),
  '/data': () => import('./views/data.js'),
  '/settings': () => import('./views/settings.js'),
  '/system': () => import('./views/system.js'),
};

let current = null;

export async function route() {
  const [path, query = ''] = (location.hash.slice(1) || '/session').split('?');
  const load = routes[path] || routes['/session'];
  current?.leave?.();
  const view = document.getElementById('view');
  view.replaceChildren();
  document.querySelectorAll('.app-nav a').forEach((a) => a.classList.toggle('active', a.getAttribute('href') === `#${path}`));
  try {
    current = await load();
    await current.render(view, new URLSearchParams(query));
  } catch (e) {
    showError(e);
  }
}

// doc/programs/web_server.md#admin-password
onAuthRequired(async () => {
  const input = h('input', { type: 'password', autocomplete: 'current-password' });
  const ok = await dialog('Admin password', h('label', { class: 'field' }, 'Password', input), [['Cancel', false], ['Log in', true, 'primary']]);
  if (!ok) return false;
  try {
    await post('/api/admin/login', { password: input.value });
    return true;
  } catch (e) {
    showError(e);
    return false;
  }
});

// doc/programs/web_server.md#settings-view
onSettingNotSet(async (name) => {
  if (!name) return false;
  const settings = (await get('/api/settings')).settings;
  const s = settings.find((x) => x.name === name);
  const input = h('input', { value: s?.value || '' });
  const ok = await dialog(`${s?.display_name || name} is not set`, h('label', { class: 'field' }, s?.display_name || name, input), [['Cancel', false], ['Save', true, 'primary']]);
  if (!ok) return false;
  await put(`/api/settings/${encodeURIComponent(name)}`, { value: input.value, active: true });
  return true;
});

window.addEventListener('hashchange', route);
connectEvents();
route();
