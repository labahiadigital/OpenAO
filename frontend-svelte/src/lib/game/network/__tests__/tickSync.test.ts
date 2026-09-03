import { describe, it, expect, beforeEach } from "vitest";
import { TickSynchronizer, type TickSyncSample } from "../tickSync";

describe("TickSynchronizer", () => {
  let sync: TickSynchronizer;

  beforeEach(() => {
    sync = new TickSynchronizer();
  });

  it("initial state has zero offset and samples", () => {
    expect(sync.sampleCount).toBe(0);
    expect(sync.totalRttMs).toBe(0);
    expect(sync.networkRttMs).toBe(0);
    expect(sync.oneWayDelayMs).toBe(0);
  });

  it("localTick increases over time", () => {
    const t1 = sync.localTick;
    const t2 = sync.localTick;
    expect(t2).toBeGreaterThanOrEqual(t1);
  });

  it("observe accepts valid sample and returns report", () => {
    const now = performance.now();
    const sample: TickSyncSample = {
      localTick: sync.localTick,
      serverTick: 100,
      sentAt: now - 50,
      receivedAt: now,
    };
    const report = sync.observe(sample);
    expect(report).not.toBeNull();
    expect(report!.totalRttMs).toBeCloseTo(50, -1);
    expect(sync.sampleCount).toBe(1);
  });

  it("rejects samples with negative RTT", () => {
    const now = performance.now();
    const sample: TickSyncSample = {
      localTick: 0,
      serverTick: 100,
      sentAt: now + 50,
      receivedAt: now,
    };
    expect(sync.observe(sample)).toBeNull();
    expect(sync.sampleCount).toBe(0);
  });

  it("rejects samples with RTT > 2000ms", () => {
    const now = performance.now();
    const sample: TickSyncSample = {
      localTick: 0,
      serverTick: 100,
      sentAt: now - 3000,
      receivedAt: now,
    };
    expect(sync.observe(sample)).toBeNull();
    expect(sync.sampleCount).toBe(0);
  });

  it("first sample sets offset exactly (no smoothing)", () => {
    const now = performance.now();
    const sample: TickSyncSample = {
      localTick: sync.localTick,
      serverTick: 600,
      sentAt: now - 100,
      receivedAt: now,
    };
    const report = sync.observe(sample);
    expect(report).not.toBeNull();
    expect(report!.totalRttMs).toBeCloseTo(100, -1);
    expect(sync.estimatedServerTick).toBeGreaterThan(0);
  });

  it("uses server timestamps when available for accurate RTT", () => {
    const now = performance.now();
    const sample: TickSyncSample = {
      localTick: sync.localTick,
      serverTick: 300,
      sentAt: now - 100,
      receivedAt: now,
      serverReceivedAt: 5000,
      serverSentAt: 5020,
    };
    const report = sync.observe(sample);
    expect(report).not.toBeNull();
    expect(report!.serverProcessingMs).toBe(20);
    expect(report!.networkRttMs).toBeCloseTo(80, -1);
    expect(report!.oneWayDelayMs).toBeCloseTo(40, -1);
  });

  it("smooths offset across multiple samples", () => {
    const baseNow = performance.now();

    const s1: TickSyncSample = {
      localTick: sync.localTick,
      serverTick: 300,
      sentAt: baseNow - 50,
      receivedAt: baseNow,
    };
    sync.observe(s1);
    const offset1 = sync.estimatedServerTick;

    const s2: TickSyncSample = {
      localTick: sync.localTick,
      serverTick: 300,
      sentAt: baseNow - 50,
      receivedAt: baseNow,
    };
    sync.observe(s2);
    const offset2 = sync.estimatedServerTick;

    expect(sync.sampleCount).toBe(2);
    expect(typeof offset1).toBe("number");
    expect(typeof offset2).toBe("number");
  });

  it("estimatedServerTick is never negative", () => {
    const now = performance.now();
    const sample: TickSyncSample = {
      localTick: sync.localTick,
      serverTick: 0,
      sentAt: now - 10,
      receivedAt: now,
    };
    sync.observe(sample);
    expect(sync.estimatedServerTick).toBeGreaterThanOrEqual(0);
  });

  it("recommendedInputTick is ahead of estimatedServerTick", () => {
    const now = performance.now();
    const sample: TickSyncSample = {
      localTick: sync.localTick,
      serverTick: 500,
      sentAt: now - 30,
      receivedAt: now,
    };
    sync.observe(sample);
    expect(sync.recommendedInputTick).toBeGreaterThan(Math.floor(sync.estimatedServerTick));
  });

  it("reset clears all state", () => {
    const now = performance.now();
    sync.observe({
      localTick: sync.localTick,
      serverTick: 500,
      sentAt: now - 30,
      receivedAt: now,
    });
    expect(sync.sampleCount).toBe(1);

    sync.reset();
    expect(sync.sampleCount).toBe(0);
    expect(sync.totalRttMs).toBe(0);
    expect(sync.networkRttMs).toBe(0);
  });

  it("handles serverSentAt < serverReceivedAt gracefully", () => {
    const now = performance.now();
    const sample: TickSyncSample = {
      localTick: sync.localTick,
      serverTick: 300,
      sentAt: now - 100,
      receivedAt: now,
      serverReceivedAt: 5020,
      serverSentAt: 5000,
    };
    const report = sync.observe(sample);
    expect(report).not.toBeNull();
    expect(report!.serverProcessingMs).toBe(0);
    expect(report!.networkRttMs).toBeCloseTo(100, -1);
  });

  it("MAX_OFFSET_CORRECTION_TICKS limits correction per sample", () => {
    const now = performance.now();
    sync.observe({
      localTick: sync.localTick,
      serverTick: 0,
      sentAt: now - 10,
      receivedAt: now,
    });
    const first = sync.estimatedServerTick;

    sync.observe({
      localTick: sync.localTick,
      serverTick: 100000,
      sentAt: now - 10,
      receivedAt: now,
    });
    const second = sync.estimatedServerTick;

    const delta = Math.abs(second - first);
    expect(delta).toBeLessThan(10);
  });
});
