// doc/programs/web_server.md#rest-api
// doc/programs/web_server.md#error-mapping

export class ApiError extends Error {
  constructor(status, body) {
    super(body?.message ?? String(status));
    this.status = status;
    this.code = body?.code;
    this.details = body?.details;
  }
}

const hooks = { authRequired: null, settingNotSet: null };

export function onAuthRequired(fn) {
  hooks.authRequired = fn;
}

export function onSettingNotSet(fn) {
  hooks.settingNotSet = fn;
}

export const RETRIES = 2;

function newKey() {
  return globalThis.crypto?.randomUUID ? crypto.randomUUID() : `${Date.now()}-${Math.random()}`;
}

// doc/programs/web_server.md#rest-api
export async function request(method, path, body, { idempotencyKey, fetchFn = globalThis.fetch } = {}) {
  const key = idempotencyKey ?? (method === 'GET' ? null : newKey());
  const headers = {};
  if (key) headers['Idempotency-Key'] = key;
  const init = { method, headers, credentials: 'same-origin' };
  if (body instanceof FormData) init.body = body;
  else if (body !== undefined) {
    headers['Content-Type'] = 'application/json';
    init.body = JSON.stringify(body);
  }
  let lastError;
  for (let attempt = 0; attempt <= RETRIES; attempt++) {
    let res;
    try {
      res = await fetchFn(path, init);
    } catch (e) {
      lastError = e;
      continue;
    }
    const text = await res.text();
    const data = text ? JSON.parse(text) : null;
    if (res.ok) return data;
    const err = new ApiError(res.status, data);
    if (err.code === 'auth_required' && hooks.authRequired && (await hooks.authRequired())) {
      return request(method, path, body, { idempotencyKey: key, fetchFn });
    }
    if (err.code === 'setting_not_set' && hooks.settingNotSet && (await hooks.settingNotSet(err.details?.setting))) {
      return request(method, path, body, { idempotencyKey: key, fetchFn });
    }
    throw err;
  }
  throw new ApiError(0, { code: 'network', message: lastError?.message || 'network error' });
}

export const get = (path) => request('GET', path);
export const post = (path, body) => request('POST', path, body ?? {});
export const put = (path, body) => request('PUT', path, body ?? {});
export const patch = (path, body) => request('PATCH', path, body ?? {});
export const del = (path) => request('DELETE', path);
export const upload = (path, form, method = 'POST') => request(method, path, form);
