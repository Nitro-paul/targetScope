// doc/programs/web_server.md#frontend

export function h(tag, attrs = {}, ...children) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs || {})) {
    if (v == null || v === false) continue;
    if (k.startsWith('on')) e.addEventListener(k.slice(2), v);
    else if (k === 'class') e.className = v;
    else if (k in e && typeof v !== 'string') e[k] = v;
    else e.setAttribute(k, v === true ? '' : v);
  }
  for (const c of children.flat()) if (c != null && c !== false) e.append(c instanceof Node ? c : String(c));
  return e;
}

export function showError(e) {
  const box = document.getElementById('messages');
  if (!box) return;
  const msg = h('div', { class: 'message error' }, e?.message || String(e), h('button', { onclick: () => msg.remove() }, '×'));
  box.append(msg);
  setTimeout(() => msg.remove(), 8000);
}

export function showInfo(text) {
  const box = document.getElementById('messages');
  if (!box) return;
  const msg = h('div', { class: 'message info' }, text);
  box.append(msg);
  setTimeout(() => msg.remove(), 5000);
}

export function dialog(title, body, buttons) {
  return new Promise((resolve) => {
    const d = h('dialog', { class: 'dialog' }, h('h2', {}, title), body);
    const row = h('div', { class: 'buttons' });
    for (const [label, value, cls] of buttons) row.append(h('button', { class: cls || '', onclick: () => { d.close(); d.remove(); resolve(value); } }, label));
    d.append(row);
    d.addEventListener('cancel', () => resolve(null));
    document.body.append(d);
    d.showModal();
  });
}

export const confirmDialog = (title, text) => dialog(title, h('p', {}, text), [['Cancel', false], ['OK', true, 'primary']]);

export async function run(fn) {
  try {
    return await fn();
  } catch (e) {
    showError(e);
    return undefined;
  }
}

export function fmtTime(unixS) {
  return unixS ? new Date(unixS * 1000).toLocaleString() : '—';
}

export function fmtDuration(s) {
  if (s == null) return '—';
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, '0')}`;
}

export function select(items, value, onChange, { empty = null } = {}) {
  const sel = h('select', { onchange: () => onChange(sel.value === '' ? null : sel.value) });
  if (empty !== null) sel.append(h('option', { value: '' }, empty));
  for (const it of items) sel.append(h('option', { value: it.value, selected: String(it.value) === String(value ?? '') }, it.label));
  return sel;
}
