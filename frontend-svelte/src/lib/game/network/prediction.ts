/**
 * Port of Elura's PredictionBuffer to TypeScript.
 * Client-side input history with authoritative reconciliation (server
 * correction + replay of unconfirmed inputs).
 */

export interface PredictionConfig {
  historyCapacity: number;
  maxReplaySteps: number;
}

export const DEFAULT_PREDICTION_CONFIG: PredictionConfig = {
  historyCapacity: 256,
  maxReplaySteps: 64,
};

export interface PredictionFrame<I, S> {
  tick: number;
  input: I;
  predictedState: S;
}

export interface ReconciliationReport<S> {
  authoritativeTick: number;
  predictedStateAtAuthoritativeTick: S | null;
  correctedState: S;
  replayedInputs: number;
  currentTick: number;
}

export type SimulateFn<I, S> = (state: S, tick: number, input: I) => S;

export class PredictionBuffer<I, S> {
  private readonly config: PredictionConfig;
  private frames: PredictionFrame<I, S>[] = [];
  private _confirmedTick = 0;

  constructor(config: Partial<PredictionConfig> = {}) {
    this.config = { ...DEFAULT_PREDICTION_CONFIG, ...config };
  }

  get confirmedTick(): number {
    return this._confirmedTick;
  }

  get pendingLen(): number {
    return this.frames.length;
  }

  get isEmpty(): boolean {
    return this.frames.length === 0;
  }

  getFrames(): ReadonlyArray<PredictionFrame<I, S>> {
    return this.frames;
  }

  record(tick: number, input: I, predictedState: S): void {
    const previous = this.frames.length > 0
      ? this.frames[this.frames.length - 1]!.tick
      : this._confirmedTick;

    if (tick <= 0 || tick <= previous) {
      throw new Error("prediction tick must increase and be positive");
    }
    if (this.frames.length >= this.config.historyCapacity) {
      throw new Error("prediction history full");
    }

    this.frames.push({ tick, input, predictedState });
  }

  reconcile(
    authoritativeTick: number,
    authoritativeState: S,
    simulate: SimulateFn<I, S>,
  ): ReconciliationReport<S> {
    if (authoritativeTick < this._confirmedTick) {
      throw new Error("authoritative tick moved backwards");
    }

    let confirmedCount = this.frames.length;
    for (let i = 0; i < this.frames.length; i++) {
      if (this.frames[i]!.tick > authoritativeTick) {
        confirmedCount = i;
        break;
      }
    }

    const replayedInputs = this.frames.length - confirmedCount;
    if (replayedInputs > this.config.maxReplaySteps) {
      throw new Error("replay limit exceeded");
    }

    let predictedStateAtAuthoritativeTick: S | null = null;
    if (confirmedCount > 0) {
      const lastConfirmedFrame = this.frames[confirmedCount - 1]!;
      if (lastConfirmedFrame.tick === authoritativeTick) {
        predictedStateAtAuthoritativeTick = lastConfirmedFrame.predictedState;
      }
    }

    this.frames.splice(0, confirmedCount);

    let correctedState = authoritativeState;
    for (const frame of this.frames) {
      correctedState = simulate(correctedState, frame.tick, frame.input);
      frame.predictedState = correctedState;
    }

    const currentTick = this.frames.length > 0
      ? this.frames[this.frames.length - 1]!.tick
      : authoritativeTick;

    this._confirmedTick = authoritativeTick;

    return {
      authoritativeTick,
      predictedStateAtAuthoritativeTick,
      correctedState,
      replayedInputs,
      currentTick,
    };
  }

  reset(confirmedTick: number): void {
    this.frames = [];
    this._confirmedTick = confirmedTick;
  }
}
