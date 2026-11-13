// doc/database.md#known-settings
// doc/programs/web_server.md#settings-view

export function checkValueRule(rule, value) {
  const r = (rule || 'text').trim();
  let m;
  if (r === 'text') return null;
  if (r.startsWith('folder')) return value.trim() ? null : 'a folder is required';
  if ((m = r.match(/^integer (-?\d+)\.\.=(-?\d+)$/))) {
    if (!/^-?\d+$/.test(value)) return 'a whole number is required';
    const v = Number(value);
    return v >= Number(m[1]) && v <= Number(m[2]) ? null : `between ${m[1]} and ${m[2]}`;
  }
  if ((m = r.match(/^one of (.+)$/))) {
    const options = m[1].split(',').map((s) => s.trim());
    return options.includes(value) ? null : `one of ${options.join(', ')}`;
  }
  if (r.startsWith('URL starting with ')) {
    const prefix = r.slice('URL starting with '.length);
    return value.startsWith(prefix) && value.length > prefix.length && !/\s/.test(value) ? null : `must start with ${prefix}`;
  }
  if (r === 'JSON object') {
    try {
      const v = JSON.parse(value);
      return v && typeof v === 'object' && !Array.isArray(v) ? null : 'a JSON object is required';
    } catch {
      return 'a JSON object is required';
    }
  }
  if (r === '<IPv4 address>:<port>') {
    m = value.match(/^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3}):(\d{1,5})$/);
    const ok = m && m.slice(1, 5).every((p) => Number(p) <= 255) && Number(m[5]) <= 65535;
    return ok ? null : 'IPv4 address and port, e.g. 0.0.0.0:8080';
  }
  if (r.startsWith('Argon2id hash')) return /^\$argon2id\$v=\d+\$m=[^$]+\$[^$]+\$[^$]+$/.test(value) ? null : 'set it under admin password';
  return null;
}
