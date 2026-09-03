import { describe, it, expect, afterAll } from "vitest";
import WebSocket from "ws";
import { PacketWriter, SERVER_PACKET_ID } from "@openao/protocol";

const WS_URL = process.env.GAME_WS_URL || "ws://localhost:7666";

describe("Combat", () => {
  let ws: WebSocket;

  afterAll(() => {
    ws?.close();
  });

  it("should accept attack packets without error", async () => {
    ws = new WebSocket(WS_URL);
    ws.binaryType = "arraybuffer";

    await new Promise<void>((resolve) => ws.on("open", resolve));

    const meleeWriter = new PacketWriter(SERVER_PACKET_ID.attackMele);
    ws.send(meleeWriter.toArrayBuffer());

    const rangeWriter = new PacketWriter(SERVER_PACKET_ID.attackRange);
    ws.send(rangeWriter.toArrayBuffer());

    const spellWriter = new PacketWriter(SERVER_PACKET_ID.attackSpell);
    spellWriter.writeByte(1);
    ws.send(spellWriter.toArrayBuffer());

    await new Promise<void>((resolve) => setTimeout(resolve, 200));
    expect(ws.readyState).toBe(WebSocket.OPEN);
  });

  it("should accept dialog packet (commands)", async () => {
    ws = new WebSocket(WS_URL);
    ws.binaryType = "arraybuffer";

    await new Promise<void>((resolve) => ws.on("open", resolve));

    const writer = new PacketWriter(SERVER_PACKET_ID.dialog);
    writer.writeString("/online");
    ws.send(writer.toArrayBuffer());

    await new Promise<void>((resolve) => setTimeout(resolve, 200));
    expect(ws.readyState).toBe(WebSocket.OPEN);
  });
});
