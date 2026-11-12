// fetch wrapper: JSON bodies, Idempotency-Key on writes (same key when resending after a
// network error), error objects { code, message, details }, 401 auth_required → password dialog.

export class ApiError extends Error {
  constructor(status, body) {
    super(body?.message ?? String(status));
    this.status = status;
    this.code = body?.code;
    this.details = body?.details;
  }
}

export async function request(method, path, body, { idempotencyKey } = {}) {
  // TODO
  throw new Error('not implemented');
}

export const get = (path) => request('GET', path);
export const post = (path, body) => request('POST', path, body, { idempotencyKey: crypto.randomUUID() });
export const put = (path, body) => request('PUT', path, body, { idempotencyKey: crypto.randomUUID() });
export const patch = (path, body) => request('PATCH', path, body, { idempotencyKey: crypto.randomUUID() });
export const del = (path) => request('DELETE', path, undefined, { idempotencyKey: crypto.randomUUID() });
