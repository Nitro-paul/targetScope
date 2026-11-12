// Router (hash-based) and navigation.
import { connectEvents } from './events.js';

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

export async function route() {
  // TODO: parse location.hash (path + query for the filter), load the view, call its render(el, params)
}

window.addEventListener('hashchange', route);
connectEvents();
route();
