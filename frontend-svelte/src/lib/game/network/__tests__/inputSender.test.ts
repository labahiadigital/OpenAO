import { describe, it, expect } from "vitest";
import { InputSender } from "../inputSender";

interface Input {
  heading: number;
}

describe("InputSender", () => {
  it("record and packet roundtrip", () => {
    const sender = new InputSender<Input>({ redundancy: 3 });
    sender.record(1, { heading: 4 });
    sender.record(2, { heading: 4 });
    const pkt = sender.packet(100);
    expect(pkt.inputs).toHaveLength(2);
    expect(pkt.inputs[0]!.targetTick).toBe(1);
    expect(pkt.inputs[1]!.targetTick).toBe(2);
    expect(pkt.clientTick).toBe(100);
  });

  it("redundancy window limits inputs in packet", () => {
    const sender = new InputSender<Input>({ redundancy: 2 });
    sender.record(1, { heading: 1 });
    sender.record(2, { heading: 2 });
    sender.record(3, { heading: 3 });
    sender.record(4, { heading: 4 });
    const pkt = sender.packet(100);
    expect(pkt.inputs).toHaveLength(2);
    expect(pkt.inputs[0]!.targetTick).toBe(3);
    expect(pkt.inputs[1]!.targetTick).toBe(4);
  });

  it("acknowledge removes confirmed inputs", () => {
    const sender = new InputSender<Input>();
    const seq1 = sender.record(1, { heading: 4 });
    const seq2 = sender.record(2, { heading: 4 });
    sender.record(3, { heading: 4 });
    expect(sender.pendingLen).toBe(3);
    const removed = sender.acknowledge({ serverTick: 1, acknowledgedSequence: seq2 });
    expect(removed).toBe(2);
    expect(sender.pendingLen).toBe(1);
    expect(sender.acknowledgedSequence).toBe(seq2);
  });

  it("cumulative ACK is idempotent", () => {
    const sender = new InputSender<Input>();
    const seq1 = sender.record(1, { heading: 4 });
    sender.record(2, { heading: 4 });
    sender.acknowledge({ serverTick: 1, acknowledgedSequence: seq1 });
    const removed = sender.acknowledge({ serverTick: 1, acknowledgedSequence: seq1 });
    expect(removed).toBe(0);
  });

  it("sequence tracking is monotonic", () => {
    const sender = new InputSender<Input>();
    const s1 = sender.record(1, { heading: 4 });
    const s2 = sender.record(2, { heading: 4 });
    const s3 = sender.record(3, { heading: 4 });
    expect(s1).toBe(1);
    expect(s2).toBe(2);
    expect(s3).toBe(3);
  });

  it("capacity limit throws", () => {
    const sender = new InputSender<Input>({ historyCapacity: 2 });
    sender.record(1, { heading: 4 });
    sender.record(2, { heading: 4 });
    expect(() => sender.record(3, { heading: 4 })).toThrow("input history full");
  });

  it("invalid ack throws", () => {
    const sender = new InputSender<Input>();
    sender.record(1, { heading: 4 });
    expect(() => sender.acknowledge({ serverTick: 1, acknowledgedSequence: 999 })).toThrow("invalid acknowledgement");
  });

  it("reset clears all state", () => {
    const sender = new InputSender<Input>();
    sender.record(1, { heading: 4 });
    sender.record(2, { heading: 4 });
    sender.reset();
    expect(sender.isEmpty).toBe(true);
    expect(sender.pendingLen).toBe(0);
    expect(sender.acknowledgedSequence).toBe(0);
  });

  it("isEmpty and pendingLen reflect state", () => {
    const sender = new InputSender<Input>();
    expect(sender.isEmpty).toBe(true);
    expect(sender.pendingLen).toBe(0);
    sender.record(1, { heading: 4 });
    expect(sender.isEmpty).toBe(false);
    expect(sender.pendingLen).toBe(1);
  });
});
