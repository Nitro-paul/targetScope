// doc/programs/web_server.md#camera-settings-menu

import * as api from '../api.js';
import { h, run, select, showInfo } from '../ui.js';

const RATE_MS = 200;

export async function render(el) {
  const [status, settings] = await Promise.all([api.get('/api/camera').catch(() => null), api.get('/api/settings')]);
  const setting = (n) => settings.settings.find((s) => s.name === n);
  const save = (name, value, active = true) => run(() => api.put(`/api/settings/${name}`, { value: String(value), active })).then(() => render(el));
  const source = setting('camera_source')?.value || 'v4l2';
  const parts = [h('h1', {}, 'Camera'),
    h('p', {}, status ? `State: ${status.state}${status.device ? ` (${status.device})` : ''}${status.width_px ? `, ${status.width_px} × ${status.height_px} @ ${status.fps} fps` : ''}` : 'camera_reader isn\'t answering.'),
    h('label', { class: 'field' }, 'Source', select([{ value: 'v4l2', label: 'USB camera' }, { value: 'rtsp', label: 'Network camera (RTSP)' }, { value: 'none', label: 'None' }], source, (v) => save('camera_source', v)))];
  if (source === 'v4l2') {
    const devs = (await api.get('/api/camera/devices').catch(() => ({ devices: [] }))).devices;
    const current = devs.find((d) => d.bus_info === setting('camera_device')?.value) || devs[0];
    parts.push(h('label', { class: 'field' }, 'Camera', select(devs.map((d) => ({ value: d.bus_info, label: `${d.name} (${d.path})` })), current?.bus_info, (v) => save('camera_device', v), { empty: 'first camera found' })));
    if (current) {
      const formats = current.formats.flatMap((f) => f.fps.map((fps) => ({ value: `${f.width_px}x${f.height_px}@${fps}`, label: `${f.width_px} × ${f.height_px} @ ${fps} fps` })));
      const now = `${setting('camera_width_px')?.value}x${setting('camera_height_px')?.value}@${setting('camera_fps')?.value}`;
      parts.push(h('label', { class: 'field' }, 'Resolution and frame rate', select(formats, now, async (v) => {
        const [wh, fps] = v.split('@');
        const [w, hh] = wh.split('x');
        await run(() => api.put('/api/settings/camera_width_px', { value: w, active: true }));
        await run(() => api.put('/api/settings/camera_height_px', { value: hh, active: true }));
        save('camera_fps', fps);
      })));
    }
  } else if (source === 'rtsp') {
    const url = h('input', { value: setting('camera_rtsp_url')?.value || '', placeholder: 'rtsp://…' });
    parts.push(h('div', { class: 'row' }, h('label', { class: 'field' }, 'RTSP URL', url), h('button', { onclick: () => save('camera_rtsp_url', url.value) }, 'Save')));
  }
  parts.push(h('div', { class: 'layout' }, h('div', { class: 'main' }, h('img', { class: 'feed', src: `/stream/raw.mjpg?${Date.now()}`, alt: 'Camera image' })), h('aside', {}, await controls(), await lens(el))));
  el.replaceChildren(...parts);
}

// doc/programs/web_server.md#camera-settings-menu
async function controls() {
  let list;
  try {
    list = (await api.get('/api/camera/controls')).controls;
  } catch {
    return h('p', { class: 'muted' }, 'No controls available');
  }
  if (!list.length) return h('p', { class: 'muted' }, 'No controls available');
  const box = h('div', { class: 'controls' });
  let last = 0;
  let pending = null;
  const send = (id, value) => {
    const go = async () => {
      last = Date.now();
      const r = await run(() => api.put(`/api/camera/controls/${id}`, { value }));
      if (r) draw(r.controls);
    };
    clearTimeout(pending);
    const wait = RATE_MS - (Date.now() - last);
    if (wait <= 0) go();
    else pending = setTimeout(go, wait);
  };
  const draw = (controls) => {
    box.replaceChildren(...controls.map((c) => {
      let input;
      if (c.kind === 'boolean') input = h('input', { type: 'checkbox', checked: !!c.value, onchange: (e) => send(c.id, e.target.checked ? 1 : 0) });
      else if (c.kind === 'menu') input = h('select', { onchange: (e) => send(c.id, Number(e.target.value)) }, c.menu.map((m) => h('option', { value: m.value, selected: m.value === c.value }, m.label)));
      else if (c.kind === 'button') input = h('button', { onclick: () => send(c.id, 1) }, 'Run');
      else input = h('input', { type: 'range', min: c.min, max: c.max, step: c.step || 1, value: c.value, oninput: (e) => send(c.id, Number(e.target.value)) });
      return h('div', { class: `control ${c.inactive ? 'inactive' : ''}` }, h('span', {}, c.name), input,
        c.kind === 'integer' ? h('button', { class: 'small', onclick: () => send(c.id, c.default) }, 'default') : null);
    }));
  };
  draw(list);
  return box;
}

// doc/programs/web_server.md#camera-settings-menu
async function lens(el) {
  const s = await api.get('/api/camera/lens').catch(() => null);
  if (!s) return h('p', { class: 'muted' }, 'Lens calibration not available.');
  const box = h('div', { class: 'card' }, h('h2', {}, 'Lens calibration'));
  box.append(h('p', {}, s.calibrated ? `Calibrated, RMS error ${s.rms_px?.toFixed(2)} px${s.applied ? '' : ' (for another resolution: calibrate again)'}` : 'Not calibrated'));
  if (s.collecting_views != null) {
    box.append(h('p', {}, `${s.collecting_views} views (at least 10; cover the image corners, use different angles)`),
      h('button', { onclick: async () => { const r = await run(() => api.post('/api/camera/lens/capture')); if (r) showInfo(r.found ? 'Checkerboard found' : 'No checkerboard found'); render(el); } }, 'Capture view'),
      h('button', { class: 'primary', onclick: async () => { const r = await run(() => api.post('/api/camera/lens/finish')); if (r) showInfo(`RMS error ${r.rms_px.toFixed(2)} px ${r.rms_px < 0.5 ? '(good)' : r.rms_px < 1 ? '(acceptable)' : '(poor)'}`); render(el); } }, 'Finish'),
      h('button', { onclick: async () => { await run(() => api.post('/api/camera/lens/cancel')); render(el); } }, 'Cancel'));
  } else {
    box.append(h('p', {}, h('a', { href: '/calibration/checkerboard_9x6.pdf', target: '_blank' }, 'Checkerboard to print')),
      h('button', { onclick: async () => { await run(() => api.post('/api/camera/lens/start')); render(el); } }, 'Calibrate lens'));
    if (s.calibrated) box.append(h('button', { class: 'danger', onclick: async () => { await run(() => api.del('/api/camera/lens')); render(el); } }, 'Delete calibration'));
  }
  return box;
}
