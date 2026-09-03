import { describe, it, expect } from "vitest";
import { InterpolationBuffer } from "../interpolation";

interface Pos {
  x: number;
  y: number;
}

describe("InterpolationBuffer", () => {
  it("insert and sample roundtrip", () => {
    const buf = new InterpolationBuffer<Pos>({ baseDelayTicks: 0, minDelayTicks: 0 });
    buf.insert(1, { x: 0, y: 0 }, 0);
    buf.insert(2, { x: 10, y: 20 }, 16);
    const s = buf.sample(1.5);
    expect(s).not.toBeNull();
    expect(s!.alpha).toBeCloseTo(0.5, 1);
    expect(s!.previous).toEqual({ x: 0, y: 0 });
    expect(s!.next).toEqual({ x: 10, y: 20 });
  });

  it("sample at exact tick returns alpha 0 on prev", () => {
    const buf = new InterpolationBuffer<Pos>({ baseDelayTicks: 0, minDelayTicks: 0 });
    buf.insert(1, { x: 0, y: 0 }, 0);
    buf.insert(2, { x: 10, y: 0 }, 16);
    const s = buf.sample(1);
    expect(s).not.toBeNull();
    expect(s!.alpha).toBe(0);
  });

  it("holdingNewest when sampling beyond newest", () => {
    const buf = new InterpolationBuffer<Pos>({ baseDelayTicks: 0, minDelayTicks: 0 });
    buf.insert(1, { x: 0, y: 0 }, 0);
    buf.insert(2, { x: 5, y: 5 }, 16);
    const s = buf.sample(10);
    expect(s).not.toBeNull();
    expect(s!.holdingNewest).toBe(true);
    expect(s!.previous).toEqual({ x: 5, y: 5 });
  });

  it("returns null on empty buffer", () => {
    const buf = new InterpolationBuffer<Pos>();
    expect(buf.sample(1)).toBeNull();
    expect(buf.isEmpty).toBe(true);
  });

  it("late insertion is tracked", () => {
    const buf = new InterpolationBuffer<Pos>();
    buf.insert(5, { x: 5, y: 0 }, 0);
    buf.insert(10, { x: 10, y: 0 }, 16);
    const result = buf.insert(7, { x: 7, y: 0 }, 32);
    expect(result).toBe("late");
    expect(buf.stats().lateSamples).toBe(1);
  });

  it("replaced insertion is tracked", () => {
    const buf = new InterpolationBuffer<Pos>();
    buf.insert(5, { x: 5, y: 0 }, 0);
    const result = buf.insert(5, { x: 99, y: 0 }, 16);
    expect(result).toBe("replaced");
    expect(buf.stats().replacedSamples).toBe(1);
  });

  it("capacity eviction drops oldest", () => {
    const buf = new InterpolationBuffer<Pos>({ capacity: 3 });
    buf.insert(1, { x: 1, y: 0 }, 0);
    buf.insert(2, { x: 2, y: 0 }, 16);
    buf.insert(3, { x: 3, y: 0 }, 32);
    buf.insert(4, { x: 4, y: 0 }, 48);
    expect(buf.length).toBe(3);
    const s = buf.sample(1);
    expect(s).not.toBeNull();
    expect(s!.previous.x).toBe(2);
  });

  it("reset clears all state", () => {
    const buf = new InterpolationBuffer<Pos>();
    buf.insert(1, { x: 0, y: 0 }, 0);
    buf.insert(2, { x: 5, y: 5 }, 16);
    expect(buf.isEmpty).toBe(false);
    buf.reset();
    expect(buf.isEmpty).toBe(true);
    expect(buf.sample(1)).toBeNull();
  });

  it("adaptive delay increases with jitter", () => {
    const buf = new InterpolationBuffer<Pos>({ baseDelayTicks: 2, smoothing: 0.5 });
    const baseDelay = buf.delayTicks;
    buf.insert(1, { x: 0, y: 0 }, 0);
    buf.insert(2, { x: 1, y: 0 }, 500);
    expect(buf.delayTicks).toBeGreaterThanOrEqual(baseDelay);
  });
});
