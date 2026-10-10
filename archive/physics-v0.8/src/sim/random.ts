export class Random {
  constructor(public state: number) { this.state = state >>> 0; }
  next() {
    this.state = (this.state + 0x6D2B79F5) >>> 0;
    let t = this.state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  }
  between(a: number, b: number) { return a + (b - a) * this.next(); }
  normal() { return Math.sqrt(-2 * Math.log(Math.max(1e-9, this.next()))) * Math.cos(2 * Math.PI * this.next()); }
}
export const clamp = (v: number, min = 0, max = 1) => Math.max(min, Math.min(max, v));
export function hash(x: number, z: number, seed: number) {
  let n = Math.imul(x, 374761393) ^ Math.imul(z, 668265263) ^ seed;
  n = Math.imul(n ^ (n >>> 13), 1274126177);
  return ((n ^ (n >>> 16)) >>> 0) / 4294967295;
}
export function noise(x: number, z: number, seed: number) {
  const ix = Math.floor(x), iz = Math.floor(z);
  let fx = x - ix, fz = z - iz;
  fx = fx * fx * (3 - 2 * fx); fz = fz * fz * (3 - 2 * fz);
  return (hash(ix, iz, seed) * (1 - fx) + hash(ix + 1, iz, seed) * fx) * (1 - fz)
    + (hash(ix, iz + 1, seed) * (1 - fx) + hash(ix + 1, iz + 1, seed) * fx) * fz;
}
export function elevation(x: number, z: number, seed: number) {
  const radius = Math.hypot(x, z);
  return noise(x * .065 + 20, z * .065 + 20, seed) * 5.2
    + noise(x * .16, z * .16, seed + 72) * 1.2 - 2.15 - Math.pow(radius / 31, 4) * 4;
}

export function habitatY(x:number,z:number,radius:number,seed:number){
  const floor=elevation(x,z,seed);return floor>.12?floor+radius:.12+radius*(1-2*.78);
}
