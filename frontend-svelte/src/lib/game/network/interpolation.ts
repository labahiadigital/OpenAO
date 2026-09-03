/**
 * Port of Elura's InterpolationBuffer to TypeScript.
 * Tick-ordered remote state buffer with adaptive jitter delay for smooth
 * rendering of remote entities.
 */

export interface InterpolationConfig {
  tickRate: number;
  capacity: number;
  baseDelayTicks: number;
  minDelayTicks: number;
  maxDelayTicks: number;
  smoothing: number;
  jitterMultiplier: number;
  lateSamplePenaltyTicks: number;
  maxAdjustmentPerSampleTicks: number;
}

export const DEFAULT_INTERPOLATION_CONFIG: InterpolationConfig = {
  tickRate: 60,
  capacity: 128,
  baseDelayTicks: 2.0,
  minDelayTicks: 1.0,
  maxDelayTicks: 8.0,
  smoothing: 0.1,
  jitterMultiplier: 2.0,
  lateSamplePenaltyTicks: 2.0,
  maxAdjustmentPerSampleTicks: 0.25,
};

export type InterpolationInsert = "newest" | "late" | "replaced";

export interface InterpolationSample<S> {
  renderTick: number;
  previousTick: number;
  previous: S;
  nextTick: number;
  next: S;
  alpha: number;
  holdingNewest: boolean;
}

export interface InterpolationStats {
  jitterTicks: number;
  lateSamplePressure: number;
  delayTicks: number;
  lateSamples: number;
  replacedSamples: number;
}

interface BufferedState<S> {
  tick: number;
  state: S;
}

export class InterpolationBuffer<S> {
  private readonly config: InterpolationConfig;
  private states: BufferedState<S>[] = [];
  private lastArrivalMs: number | null = null;
  private lastNewest: { tick: number; arrivedAt: number } | null = null;
  private jitterTicks = 0;
  private latePressure = 0;
  private _delayTicks: number;
  private _lateSamples = 0;
  private _replacedSamples = 0;

  constructor(config: Partial<InterpolationConfig> = {}) {
    this.config = { ...DEFAULT_INTERPOLATION_CONFIG, ...config };
    this._delayTicks = this.config.baseDelayTicks;
  }

  get delayTicks(): number {
    return this._delayTicks;
  }

  get length(): number {
    return this.states.length;
  }

  get isEmpty(): boolean {
    return this.states.length === 0;
  }

  insert(tick: number, state: S, arrivedAtMs: number): InterpolationInsert {
    if (tick <= 0) throw new Error("interpolation tick must be positive");
    if (this.lastArrivalMs !== null && arrivedAtMs < this.lastArrivalMs) {
      throw new Error("interpolation arrival time moved backwards");
    }

    const newestTick = this.states.length > 0 ? this.states[this.states.length - 1]!.tick : 0;

    const existingIndex = this.binarySearch(tick);
    let disposition: InterpolationInsert;

    if (existingIndex >= 0) {
      this.states[existingIndex]!.state = state;
      this._replacedSamples++;
      disposition = "replaced";
    } else if (newestTick > 0 && tick < newestTick) {
      const insertAt = this.findInsertIndex(tick);
      this.states.splice(insertAt, 0, { tick, state });
      this._lateSamples++;
      disposition = "late";
    } else {
      this.states.push({ tick, state });
      disposition = "newest";
    }

    const isLate = disposition === "late" ? 1 : 0;
    this.latePressure += (isLate - this.latePressure) * this.config.smoothing;

    if (disposition === "newest") {
      if (this.lastNewest !== null) {
        const tickGap = tick - this.lastNewest.tick;
        if (tickGap > 0) {
          const actualSeconds = (arrivedAtMs - this.lastNewest.arrivedAt) / 1000;
          const expectedSeconds = tickGap / this.config.tickRate;
          const variationTicks = Math.abs(actualSeconds - expectedSeconds) * this.config.tickRate;
          this.jitterTicks += (variationTicks - this.jitterTicks) * this.config.smoothing;
        }
      }
      this.lastNewest = { tick, arrivedAt: arrivedAtMs };
    }
    this.lastArrivalMs = arrivedAtMs;

    const targetDelay = Math.min(
      this.config.maxDelayTicks,
      Math.max(
        this.config.minDelayTicks,
        this.config.baseDelayTicks +
          this.jitterTicks * this.config.jitterMultiplier +
          this.latePressure * this.config.lateSamplePenaltyTicks
      )
    );
    const adjustment = Math.min(
      this.config.maxAdjustmentPerSampleTicks,
      Math.max(-this.config.maxAdjustmentPerSampleTicks, targetDelay - this._delayTicks)
    );
    this._delayTicks += adjustment;

    while (this.states.length > this.config.capacity) {
      this.states.shift();
    }

    return disposition;
  }

  sample(estimatedServerTick: number): InterpolationSample<S> | null {
    if (this.states.length === 0) return null;

    const oldest = this.states[0]!;
    const newest = this.states[this.states.length - 1]!;
    const renderTick = Math.max(0, estimatedServerTick - this._delayTicks);

    if (renderTick <= oldest.tick) {
      return {
        renderTick,
        previousTick: oldest.tick,
        previous: oldest.state,
        nextTick: oldest.tick,
        next: oldest.state,
        alpha: 0,
        holdingNewest: false,
      };
    }

    if (renderTick >= newest.tick) {
      return {
        renderTick,
        previousTick: newest.tick,
        previous: newest.state,
        nextTick: newest.tick,
        next: newest.state,
        alpha: 0,
        holdingNewest: renderTick > newest.tick,
      };
    }

    const prevBound = Math.floor(renderTick);
    const nextBound = Math.ceil(renderTick);

    let prevIdx = this.findFloorIndex(prevBound);
    let nextIdx = this.findCeilIndex(nextBound);

    if (prevIdx < 0) prevIdx = 0;
    if (nextIdx >= this.states.length) nextIdx = this.states.length - 1;
    if (prevIdx > nextIdx) prevIdx = nextIdx;

    const prev = this.states[prevIdx]!;
    const next = this.states[nextIdx]!;
    const span = next.tick - prev.tick;
    const alpha = span === 0 ? 0 : Math.min(1, Math.max(0, (renderTick - prev.tick) / span));

    return {
      renderTick,
      previousTick: prev.tick,
      previous: prev.state,
      nextTick: next.tick,
      next: next.state,
      alpha,
      holdingNewest: false,
    };
  }

  stats(): InterpolationStats {
    return {
      jitterTicks: this.jitterTicks,
      lateSamplePressure: this.latePressure,
      delayTicks: this._delayTicks,
      lateSamples: this._lateSamples,
      replacedSamples: this._replacedSamples,
    };
  }

  reset(): void {
    this.states = [];
    this.lastArrivalMs = null;
    this.lastNewest = null;
    this.jitterTicks = 0;
    this.latePressure = 0;
    this._delayTicks = this.config.baseDelayTicks;
    this._lateSamples = 0;
    this._replacedSamples = 0;
  }

  private binarySearch(tick: number): number {
    let lo = 0, hi = this.states.length - 1;
    while (lo <= hi) {
      const mid = (lo + hi) >>> 1;
      const midTick = this.states[mid]!.tick;
      if (midTick === tick) return mid;
      if (midTick < tick) lo = mid + 1;
      else hi = mid - 1;
    }
    return -1;
  }

  private findInsertIndex(tick: number): number {
    let lo = 0, hi = this.states.length;
    while (lo < hi) {
      const mid = (lo + hi) >>> 1;
      if (this.states[mid]!.tick < tick) lo = mid + 1;
      else hi = mid;
    }
    return lo;
  }

  private findFloorIndex(tick: number): number {
    let lo = 0, hi = this.states.length - 1, result = 0;
    while (lo <= hi) {
      const mid = (lo + hi) >>> 1;
      if (this.states[mid]!.tick <= tick) {
        result = mid;
        lo = mid + 1;
      } else {
        hi = mid - 1;
      }
    }
    return result;
  }

  private findCeilIndex(tick: number): number {
    let lo = 0, hi = this.states.length - 1, result = this.states.length - 1;
    while (lo <= hi) {
      const mid = (lo + hi) >>> 1;
      if (this.states[mid]!.tick >= tick) {
        result = mid;
        hi = mid - 1;
      } else {
        lo = mid + 1;
      }
    }
    return result;
  }
}
