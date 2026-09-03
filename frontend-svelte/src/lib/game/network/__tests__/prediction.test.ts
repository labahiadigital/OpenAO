import { describe, it, expect } from "vitest";
import { PredictionBuffer, type SimulateFn } from "../prediction";

interface Input {
  heading: number;
}

interface State {
  x: number;
  y: number;
}

const simulate: SimulateFn<Input, State> = (state, _tick, input) => {
  const dx = input.heading === 4 ? 1 : input.heading === 3 ? -1 : 0;
  const dy = input.heading === 2 ? 1 : input.heading === 1 ? -1 : 0;
  return { x: state.x + dx, y: state.y + dy };
};

describe("PredictionBuffer", () => {
  it("record and reconcile roundtrip (server agrees)", () => {
    const buf = new PredictionBuffer<Input, State>();
    buf.record(1, { heading: 4 }, { x: 1, y: 0 });
    buf.record(2, { heading: 4 }, { x: 2, y: 0 });
    const report = buf.reconcile(1, { x: 1, y: 0 }, simulate);
    expect(report.correctedState).toEqual({ x: 2, y: 0 });
    expect(report.replayedInputs).toBe(1);
    expect(buf.pendingLen).toBe(1);
  });

  it("replay corrects misprediction", () => {
    const buf = new PredictionBuffer<Input, State>();
    buf.record(1, { heading: 4 }, { x: 1, y: 0 });
    buf.record(2, { heading: 4 }, { x: 2, y: 0 });
    buf.record(3, { heading: 4 }, { x: 3, y: 0 });
    const report = buf.reconcile(1, { x: 0, y: 0 }, simulate);
    expect(report.correctedState).toEqual({ x: 2, y: 0 });
    expect(report.replayedInputs).toBe(2);
  });

  it("reconcile all inputs leaves empty", () => {
    const buf = new PredictionBuffer<Input, State>();
    buf.record(1, { heading: 4 }, { x: 1, y: 0 });
    buf.record(2, { heading: 4 }, { x: 2, y: 0 });
    const report = buf.reconcile(2, { x: 2, y: 0 }, simulate);
    expect(report.replayedInputs).toBe(0);
    expect(buf.isEmpty).toBe(true);
    expect(report.correctedState).toEqual({ x: 2, y: 0 });
  });

  it("backwards tick rejection", () => {
    const buf = new PredictionBuffer<Input, State>();
    buf.record(1, { heading: 4 }, { x: 1, y: 0 });
    buf.reconcile(1, { x: 1, y: 0 }, simulate);
    expect(() => buf.reconcile(0, { x: 0, y: 0 }, simulate)).toThrow("authoritative tick moved backwards");
  });

  it("reject non-increasing record ticks", () => {
    const buf = new PredictionBuffer<Input, State>();
    buf.record(2, { heading: 4 }, { x: 1, y: 0 });
    expect(() => buf.record(1, { heading: 4 }, { x: 0, y: 0 })).toThrow();
    expect(() => buf.record(2, { heading: 4 }, { x: 0, y: 0 })).toThrow();
  });

  it("capacity limit throws", () => {
    const buf = new PredictionBuffer<Input, State>({ historyCapacity: 2 });
    buf.record(1, { heading: 4 }, { x: 1, y: 0 });
    buf.record(2, { heading: 4 }, { x: 2, y: 0 });
    expect(() => buf.record(3, { heading: 4 }, { x: 3, y: 0 })).toThrow("prediction history full");
  });

  it("reset clears state", () => {
    const buf = new PredictionBuffer<Input, State>();
    buf.record(1, { heading: 4 }, { x: 1, y: 0 });
    buf.record(2, { heading: 4 }, { x: 2, y: 0 });
    buf.reset(0);
    expect(buf.isEmpty).toBe(true);
    expect(buf.confirmedTick).toBe(0);
  });

  it("confirmed tick advances after reconcile", () => {
    const buf = new PredictionBuffer<Input, State>();
    expect(buf.confirmedTick).toBe(0);
    buf.record(1, { heading: 4 }, { x: 1, y: 0 });
    buf.record(2, { heading: 4 }, { x: 2, y: 0 });
    buf.reconcile(1, { x: 1, y: 0 }, simulate);
    expect(buf.confirmedTick).toBe(1);
  });
});
