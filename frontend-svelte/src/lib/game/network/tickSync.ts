const TICK_RATE = 60;
const INPUT_DELAY_TICKS = 2;
const SMOOTHING = 0.2;
const MAX_RTT_MS = 2000;
const MAX_OFFSET_CORRECTION_TICKS = 4.0;

export interface TickSyncSample {
  localTick: number;
  serverTick: number;
  sentAt: number;
  receivedAt: number;
  serverReceivedAt?: number;
  serverSentAt?: number;
}

export interface TickSyncReport {
  totalRttMs: number;
  networkRttMs: number;
  serverProcessingMs: number;
  oneWayDelayMs: number;
  offsetTicks: number;
  estimatedServerTick: number;
  recommendedInputTick: number;
}

/**
 * Client-side tick synchronizer following Elura's TickSynchronizer API.
 * Estimates the authoritative server tick from heartbeat probe round-trips.
 * When server_received_at/server_sent_at are available, separates network RTT
 * from server processing time for a more accurate offset estimate.
 */
export class TickSynchronizer {
  private offsetTicks = 0;
  private smoothedTotalRttMs = 0;
  private smoothedNetworkRttMs = 0;
  private samples = 0;
  private startTime = performance.now();

  get localTick(): number {
    return ((performance.now() - this.startTime) / 1000) * TICK_RATE;
  }

  get estimatedServerTick(): number {
    return Math.max(0, this.localTick + this.offsetTicks);
  }

  get recommendedInputTick(): number {
    const estimated = Math.floor(this.estimatedServerTick);
    return estimated + INPUT_DELAY_TICKS + 1;
  }

  get totalRttMs(): number {
    return this.smoothedTotalRttMs;
  }

  get networkRttMs(): number {
    return this.smoothedNetworkRttMs;
  }

  get oneWayDelayMs(): number {
    return this.smoothedNetworkRttMs / 2;
  }

  get sampleCount(): number {
    return this.samples;
  }

  observe(sample: TickSyncSample): TickSyncReport | null {
    const totalRttMs = sample.receivedAt - sample.sentAt;
    if (totalRttMs < 0 || totalRttMs > MAX_RTT_MS) {
      return null;
    }

    let serverProcessingMs = 0;
    let networkRttMs = totalRttMs;

    if (
      sample.serverReceivedAt !== undefined &&
      sample.serverSentAt !== undefined &&
      sample.serverSentAt >= sample.serverReceivedAt
    ) {
      serverProcessingMs = sample.serverSentAt - sample.serverReceivedAt;
      networkRttMs = Math.max(0, totalRttMs - serverProcessingMs);
    }

    const oneWaySeconds = (networkRttMs / 2) / 1000;
    const estimatedServerTick = sample.serverTick + oneWaySeconds * TICK_RATE;
    const rawOffset = estimatedServerTick - sample.localTick;

    if (this.samples === 0) {
      this.offsetTicks = rawOffset;
      this.smoothedTotalRttMs = totalRttMs;
      this.smoothedNetworkRttMs = networkRttMs;
    } else {
      const correction = Math.max(
        -MAX_OFFSET_CORRECTION_TICKS,
        Math.min(MAX_OFFSET_CORRECTION_TICKS, rawOffset - this.offsetTicks)
      );
      this.offsetTicks += correction * SMOOTHING;
      this.smoothedTotalRttMs += (totalRttMs - this.smoothedTotalRttMs) * SMOOTHING;
      this.smoothedNetworkRttMs += (networkRttMs - this.smoothedNetworkRttMs) * SMOOTHING;
    }
    this.samples++;

    return {
      totalRttMs,
      networkRttMs,
      serverProcessingMs,
      oneWayDelayMs: networkRttMs / 2,
      offsetTicks: this.offsetTicks,
      estimatedServerTick: this.estimatedServerTick,
      recommendedInputTick: this.recommendedInputTick,
    };
  }

  reset(): void {
    this.offsetTicks = 0;
    this.smoothedTotalRttMs = 0;
    this.smoothedNetworkRttMs = 0;
    this.samples = 0;
    this.startTime = performance.now();
  }
}
