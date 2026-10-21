// Builds the Surety mark (Low seal + three marks) as outlined SVG paths.
// usage: node assets/logo/build.js [out-dir]   (defaults to this directory)
const fs = require('fs'), path = require('path');
const OUT = process.argv[2] || __dirname;
const D = Math.PI / 180;
const rng = seed => () => { seed = seed + 0x6D2B79F5 | 0; let t = Math.imul(seed ^ seed >>> 15, 1 | seed); t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t; return ((t ^ t >>> 14) >>> 0) / 4294967296; };
const ad = (a, b) => { const d = Math.abs(((a - b) % 360 + 360) % 360); return d > 180 ? 360 - d : d; };
const f1 = x => +x.toFixed(1);
const pt = (R, a) => [100 + R * Math.sin(a * D), 100 - R * Math.cos(a * D)];

const doc = () => ({ paths: [], circles: [], pts: [] });
function circle(d, x, y, r) { if (r < 0.25) return; d.circles.push([x, y, r]); d.pts.push([x - r, y - r], [x + r, y + r]); }
function closedSmooth(P) {
  const n = P.length; let s = `M${f1(P[0][0])} ${f1(P[0][1])}`;
  for (let i = 0; i < n; i++) {
    const p0 = P[(i - 1 + n) % n], p1 = P[i], p2 = P[(i + 1) % n], p3 = P[(i + 2) % n];
    s += `C${f1(p1[0] + (p2[0] - p0[0]) / 6)} ${f1(p1[1] + (p2[1] - p0[1]) / 6)} ${f1(p2[0] - (p3[0] - p1[0]) / 6)} ${f1(p2[1] - (p3[1] - p1[1]) / 6)} ${f1(p2[0])} ${f1(p2[1])}`;
  }
  return s + 'Z';
}
function ring(d, R, w, step = 3) {
  const O = [], I = [];
  for (let a = 0; a < 360; a += step) { O.push(pt(R(a) + w(a) / 2, a)); I.push(pt(R(a) - w(a) / 2, a)); }
  d.paths.push({ d: closedSmooth(O) + closedSmooth(I.reverse()), evenodd: true }); d.pts.push(...O);
}
// a tapered stroke: same centreline and width as the dot-stamped sketch, drawn as one outline
function tendril(d, x, y, a, len, w0, curl) {
  const n = Math.ceil(len / 0.7), L = [], Rt = [];
  circle(d, x, y, w0 / 2);
  for (let i = 0; i <= n; i++) {
    const t = i / n, r = w0 * Math.pow(1 - t, 0.8) / 2, nx = Math.cos(a * D), ny = Math.sin(a * D);
    if (i % 2 == 0 || i == n) { L.push([x + nx * r, y + ny * r]); Rt.push([x - nx * r, y - ny * r]); }
    x += Math.sin(a * D) * 0.7; y -= Math.cos(a * D) * 0.7; a += curl;
  }
  const P = L.concat(Rt.reverse());
  d.paths.push('M' + P.map(p => f1(p[0]) + ' ' + f1(p[1])).join('L') + 'Z'); d.pts.push(...P);
}
function blot(d, x, y, r, rand) {
  circle(d, x, y, r);
  for (let i = 0; i < 8; i++) { const a = rand() * 360, k = r * (0.5 + rand() * 0.6); circle(d, x + Math.sin(a * D) * k, y - Math.cos(a * D) * k, r * (0.25 + rand() * 0.45)); }
}
function fringe(d, R, w, a0, a1, n, rand, len) {
  for (let i = 0; i < n; i++) {
    const a = a0 + (a1 - a0) * rand(), e = R(a) + w(a) / 2 - 1.5;
    if (i % 3 == 2) { const p = pt(e + 4 + rand() * len, a + (rand() - 0.5) * 6); circle(d, p[0], p[1], 0.8 + rand() * 1.6); continue; }
    const p = pt(e, a);
    tendril(d, p[0], p[1], a + (rand() - 0.5) * 50, 2 + rand() * len * 0.7, 2 + rand() * 3, (rand() < 0.5 ? -1 : 1) * (3 + rand() * 5));
  }
}

const S = 215, T = 40;

// the full mark: ring, seal low left, ink spill, three marks top right
function full() {
  const d = doc(), r = rng(21);
  const R = a => 58 + 1.2 * Math.sin(a * D * 3 + 0.5);
  const w = a => 8 + 11 * Math.exp(-((ad(a, S) / 24) ** 2)) + 4 * Math.exp(-((ad(a, T) / 26) ** 2)) + 1.3 * Math.sin(a * D * 7);
  const edge = a => R(a) + w(a) / 2 - 1.5;
  ring(d, R, w);
  fringe(d, R, w, S - 60, S + 45, 18, r, 14);
  let a = 30, p = pt(edge(a), a); tendril(d, p[0], p[1], a + 30, 16, 5.5, -0.6);   // type: straight
  a = 42; p = pt(edge(a), a); tendril(d, p[0], p[1], a + 28, 24, 6, -3);            // property: curls
  a = 55; p = pt(edge(a), a); tendril(d, p[0], p[1], a + 22, 8, 5, 0);              // example: short, then a dot
  p = pt(edge(a) + 15, a + 9); circle(d, p[0], p[1], 3);
  p = pt(R(S), S); blot(d, p[0], p[1], 9, r);
  for (const [rad, ang, rr] of [[82, 236, 2.6], [88, 204, 1.6], [79, 190, 1.2]]) { p = pt(rad, ang); circle(d, p[0], p[1], rr); }
  return d;
}

// 16-32px: heavier ring, the three marks merged into one flick, two large drops, no fringe
function small() {
  const d = doc();
  const R = () => 56;
  const w = a => 22 + 20 * Math.exp(-((ad(a, S) / 32) ** 2)) + 9 * Math.exp(-((ad(a, T) / 22) ** 2));
  ring(d, R, w);
  let p = pt(56 + w(T) / 2 - 3, T + 4); tendril(d, p[0], p[1], T + 34, 17, 15, -1.5);
  p = pt(64, S); circle(d, p[0], p[1], 15);
  p = pt(96, 230); circle(d, p[0], p[1], 7.5);
  return d;
}

function bbox(d) {
  let x0 = 1e9, y0 = 1e9, x1 = -1e9, y1 = -1e9;
  for (const [x, y] of d.pts) { x0 = Math.min(x0, x); y0 = Math.min(y0, y); x1 = Math.max(x1, x); y1 = Math.max(y1, y); }
  return { x0, y0, x1, y1 };
}
const STYLE = '<style>.ink{fill:#141414}@media (prefers-color-scheme:dark){.ink{fill:#f2f0ea}}</style>';
function body(d) {
  return d.paths.map(p => typeof p == 'string' ? `<path d="${p}"/>` : `<path fill-rule="evenodd" d="${p.d}"/>`).join('') + d.circles.map(([x, y, r]) => `<circle cx="${f1(x)}" cy="${f1(y)}" r="${f1(r)}"/>`).join('');
}
function markSvg(d, pad) {
  const b = bbox(d), side = Math.max(b.x1 - b.x0, b.y1 - b.y0) + 2 * pad;
  const cx = (b.x0 + b.x1) / 2, cy = (b.y0 + b.y1) / 2;
  const vb = [f1(cx - side / 2), f1(cy - side / 2), f1(side), f1(side)].join(' ');
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${vb}" role="img" aria-label="Surety"><title>Surety</title>${STYLE}<g class="ink">${body(d)}</g></svg>\n`;
}
function lockupSvg(d) {
  const b = bbox(d), pad = 4, h = b.y1 - b.y0 + 2 * pad, x = b.x0 - pad, y = b.y0 - pad;
  const fs_ = h * 0.62, tx = b.x1 + h * 0.2, ty = 100 + fs_ * 0.24, width = tx - x + fs_ * 2.95;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${f1(x)} ${f1(y)} ${f1(width)} ${f1(h)}" role="img" aria-label="Surety"><title>Surety</title>${STYLE}<g class="ink">${body(d)}<text x="${f1(tx)}" y="${f1(ty)}" font-family="'Iowan Old Style','Palatino Linotype',Palatino,Georgia,serif" font-size="${f1(fs_)}" letter-spacing="0.5">surety</text></g></svg>\n`;
}

fs.mkdirSync(OUT, { recursive: true });
fs.writeFileSync(path.join(OUT, 'surety-mark.svg'), markSvg(full(), 6));
fs.writeFileSync(path.join(OUT, 'surety-mark-small.svg'), markSvg(small(), 6));
fs.writeFileSync(path.join(OUT, 'surety-lockup.svg'), lockupSvg(full()));
