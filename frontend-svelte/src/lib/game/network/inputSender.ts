/**
 * Port of Elura's InputSender to TypeScript.
 * Client-side bounded input history with redundant packet building
 * and cumulative server acknowledgement.
 */

export interface InputSenderConfig {
  historyCapacity: number;
  redundancy: number;
}

export const DEFAULT_INPUT_SENDER_CONFIG: InputSenderConfig = {
  historyCapacity: 64,
  redundancy: 3,
};

export interface InputFrame<T> {
  sequence: number;
  targetTick: number;
  input: T;
}

export interface InputPacket<T> {
  clientTick: number;
  acknowledgedServerTick: number;
  inputs: InputFrame<T>[];
}

export interface InputAck {
  serverTick: number;
  acknowledgedSequence: number;
}

export class InputSender<T> {
  private readonly config: InputSenderConfig;
  private nextSequence: number;
  private _acknowledgedSequence: number;
  private _acknowledgedServerTick = 0;
  private history: InputFrame<T>[] = [];

  constructor(config: Partial<InputSenderConfig> = {}, nextSequence = 1) {
    this.config = { ...DEFAULT_INPUT_SENDER_CONFIG, ...config };
    if (nextSequence <= 0) throw new Error("next input sequence must be positive");
    this.nextSequence = nextSequence;
    this._acknowledgedSequence = nextSequence - 1;
  }

  get acknowledgedSequence(): number {
    return this._acknowledgedSequence;
  }

  get acknowledgedServerTick(): number {
    return this._acknowledgedServerTick;
  }

  get pendingLen(): number {
    return this.history.length;
  }

  get isEmpty(): boolean {
    return this.history.length === 0;
  }

  record(targetTick: number, input: T): number {
    if (targetTick <= 0) throw new Error("input target tick must be positive");
    if (this.history.length >= this.config.historyCapacity) {
      throw new Error("input history full");
    }

    const sequence = this.nextSequence++;
    this.history.push({ sequence, targetTick, input });
    return sequence;
  }

  acknowledge(ack: InputAck): number {
    const lastIssued = this.nextSequence - 1;
    if (ack.acknowledgedSequence > lastIssued) {
      throw new Error("invalid acknowledgement: sequence exceeds issued");
    }

    this._acknowledgedServerTick = Math.max(this._acknowledgedServerTick, ack.serverTick);

    if (ack.acknowledgedSequence <= this._acknowledgedSequence) {
      return 0;
    }

    this._acknowledgedSequence = ack.acknowledgedSequence;
    const before = this.history.length;

    while (
      this.history.length > 0 &&
      this.history[0]!.sequence <= this._acknowledgedSequence
    ) {
      this.history.shift();
    }

    return before - this.history.length;
  }

  packet(clientTick: number): InputPacket<T> {
    const skip = Math.max(0, this.history.length - this.config.redundancy);
    return {
      clientTick,
      acknowledgedServerTick: this._acknowledgedServerTick,
      inputs: this.history.slice(skip).map(f => ({ ...f })),
    };
  }

  reset(nextSequence = 1): void {
    if (nextSequence <= 0) throw new Error("next input sequence must be positive");
    this.nextSequence = nextSequence;
    this._acknowledgedSequence = nextSequence - 1;
    this._acknowledgedServerTick = 0;
    this.history = [];
  }
}
