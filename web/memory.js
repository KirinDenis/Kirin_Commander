// A drive in memory, as a page's source is: list, readBytes, write,
// remove, mkdir, rename; paths /folder/file, folders ending in /. For the
// test page and for a test, where a browser's storage would outlive it.

const utf8 = new TextEncoder();

export class MemorySource {
  /** files: { '/GAMES/README.TXT': 'text or bytes', '/EMPTY/': null } */
  constructor(files = {}) {
    this.files = new Map();     // '/A/B.TXT' -> { data, date }
    this.dirs = new Set(['/']);
    for (const [path, data] of Object.entries(files)) {
      if (path.endsWith('/')) this.mkdirSync(path);
      else this.put(path, typeof data === 'string' ? utf8.encode(data) : data);
    }
  }

  parent(path) { return path.replace(/\/?[^/]+\/?$/, '/') || '/'; }

  mkdirSync(path) {
    const p = path.endsWith('/') ? path : path + '/';
    for (let i = 1; i < p.length; i++) if (p[i] === '/') this.dirs.add(p.slice(0, i + 1));
  }

  put(path, data) {
    this.mkdirSync(this.parent(path));
    this.files.set(path, { data, date: new Date() });
  }

  find(path) {
    const p = path.replace(/\/$/, '');
    if (this.files.has(p)) return { file: p };
    if (this.dirs.has(p + '/')) return { dir: p + '/' };
    return null;
  }

  async list(dir) {
    if (!this.dirs.has(dir)) throw Object.assign(new Error(`${dir} is not here`), { name: 'NotFoundError' });
    const out = [];
    for (const d of this.dirs) {
      if (d !== dir && this.parent(d) === dir) out.push({ name: d.slice(dir.length, -1), dir: true, size: 0, date: new Date() });
    }
    for (const [p, f] of this.files) {
      if (this.parent(p) === dir) out.push({ name: p.slice(dir.length), dir: false, size: f.data.length, date: f.date });
    }
    return out;
  }

  async readBytes(path) {
    const f = this.files.get(path);
    if (!f) throw Object.assign(new Error(`${path} is not here`), { name: 'NotFoundError' });
    return f.data.slice();
  }

  async read(path) { return new TextDecoder().decode(await this.readBytes(path)); }

  async write(path, data) { this.put(path, typeof data === 'string' ? utf8.encode(data) : new Uint8Array(data)); }

  async mkdir(path) { this.mkdirSync(path); }

  async remove(path) {
    const hit = this.find(path);
    if (!hit) throw Object.assign(new Error(`${path} is not here`), { name: 'NotFoundError' });
    if (hit.file) { this.files.delete(hit.file); return; }
    for (const d of [...this.dirs]) if (d.startsWith(hit.dir)) this.dirs.delete(d);
    for (const p of [...this.files.keys()]) if (p.startsWith(hit.dir)) this.files.delete(p);
  }

  async rename(from, to) {
    const hit = this.find(from);
    if (!hit) throw Object.assign(new Error(`${from} is not here`), { name: 'NotFoundError' });
    if (hit.file) {
      const f = this.files.get(hit.file);
      this.files.delete(hit.file);
      this.put(to.replace(/\/$/, ''), f.data);
      return;
    }
    const target = to.endsWith('/') ? to : to + '/';
    for (const d of [...this.dirs]) if (d.startsWith(hit.dir)) { this.dirs.delete(d); this.dirs.add(target + d.slice(hit.dir.length)); }
    for (const [p, f] of [...this.files]) if (p.startsWith(hit.dir)) { this.files.delete(p); this.files.set(target + p.slice(hit.dir.length), f); }
    this.mkdirSync(target);
  }
}

/**
 * A letter over a source, as WIRE CITY's Drive is (site/drives.js): its
 * own folder of the source as the drive's root.
 */
export class Letter {
  constructor(letter, base, { root = '/', kind = '', title = '', readOnly = false } = {}) {
    Object.assign(this, { letter, base, root, kind, title, readOnly });
  }
  real(path) { return this.root.replace(/\/$/, '') + path; }
  list(dir) { return this.base.list(this.real(dir)); }
  readBytes(path) { return this.base.readBytes(this.real(path)); }
  write(path, data) { return this.base.write(this.real(path), data); }
  remove(path) { return this.base.remove(this.real(path)); }
  mkdir(path) { return this.base.mkdir(this.real(path)); }
  rename(from, to) { return this.base.rename(this.real(from), this.real(to)); }
}
