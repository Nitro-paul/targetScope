// doc/programs/web_server.md#frontend

import * as api from '../api.js';
import { confirmDialog, h, run, select, showInfo } from '../ui.js';
import { filterToQuery, queryToFilter, renderFilterPanel } from './filter_panel.js';

export async function render(el, params) {
  const filter = queryToFilter(params.toString());
  let format = params.get('format') || 'json';
  let images = false;
  const panel = h('div');
  const list = h('div');
  const imagesBox = h('label', { class: 'check' }, h('input', { type: 'checkbox', onchange: (e) => (images = e.target.checked) }), ' include images (ZIP)');
  const exportNow = async () => {
    const r = await run(() => api.post('/api/exports', { filter, format, include_images: images && format !== 'png' }));
    if (r) {
      showInfo(`Exported: ${r.files.join(', ')}`);
      refresh();
    }
  };
  const file = h('input', { type: 'file', accept: '.json,.zip,application/json,application/zip' });
  const importNow = async () => {
    if (!file.files[0]) return;
    const f = new FormData();
    f.append('file', file.files[0]);
    const r = await run(() => api.upload('/api/imports', f));
    if (r) showInfo(`Imported: sessions ${r.sessions.created} new, ${r.sessions.existing} existing; images ${r.images.restored}${r.warnings.length ? `; ${r.warnings.join('; ')}` : ''}`);
  };
  const refresh = async () => {
    const r = await api.get('/api/exports').catch(() => ({ files: [] }));
    list.replaceChildren(h('table', { class: 'table' }, h('tbody', {}, r.files.map((f) => h('tr', {},
      h('td', {}, h('a', { href: `/exports/${encodeURIComponent(f.name)}` }, f.name)), h('td', {}, `${(f.size_bytes / 1024).toFixed(0)} KB`),
      h('td', {}, new Date(f.created_ms).toLocaleString()),
      h('td', {}, h('button', { class: 'danger', onclick: async () => (await confirmDialog('Delete export', f.name)) && (await run(() => api.del(`/api/exports/${encodeURIComponent(f.name)}`))) && refresh() }, 'Delete')))))));
  };
  el.replaceChildren(h('h1', {}, 'Export'), panel,
    h('div', { class: 'row' }, h('label', { class: 'field' }, 'Format', select([{ value: 'json', label: 'JSON' }, { value: 'csv', label: 'CSV' }, { value: 'png', label: 'PNG target plot' }], format, (v) => (format = v))), imagesBox,
      h('button', { class: 'primary', onclick: exportNow }, 'Export')),
    h('h2', {}, 'Exports'), list,
    h('h1', {}, 'Import'), h('div', { class: 'row' }, file, h('button', { class: 'primary', onclick: importNow }, 'Import')));
  await renderFilterPanel(panel, filter, (f) => (location.hash = `#/data?${filterToQuery(f)}`));
  await refresh();
}
