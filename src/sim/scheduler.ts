/** Fixed biology ticks; high requested speed changes throughput, never dt. */
export const MAX_REQUESTED_SPEED = 10000;
export const BATCH_MS = 6;
export function requestedSpeed(value: number): number {
  if (!Number.isFinite(value)) throw new Error('Speed must be finite');
  return value === 0 ? 0 : Math.max(1, Math.min(MAX_REQUESTED_SPEED, value));
}
export class SimulationClock {
  private last = 0;
  private debt = 0;
  private rateStart = 0;
  private ticks = 0;
  actualSpeed = 0;
  reset(now: number) { this.last = this.rateStart = now; this.debt = this.ticks = this.actualSpeed = 0; }
  accrue(now: number, speed: number) {
    const seconds = Math.max(0, Math.min((now - this.last) / 1000, .25));
    this.last = now;
    // Bound outstanding work. Changing speed/pause resets debt; no old backlog.
    this.debt = Math.min(32, this.debt + seconds * speed * 4);
  }
  take(speed: number): boolean {
    if (speed === 0) return true; // Max: continuously use the bounded batch.
    if (this.debt < 1) return false;
    this.debt -= 1; return true;
  }
  record() { this.ticks++; }
  measure(now: number) {
    const elapsed = (now - this.rateStart) / 1000;
    if (elapsed >= .5) { this.actualSpeed = this.ticks * .25 / elapsed; this.rateStart = now; this.ticks = 0; }
    return this.actualSpeed;
  }
}
