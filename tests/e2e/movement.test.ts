import { describe, it, expect, afterAll } from "vitest";
import WebSocket from "ws";
import { PacketWriter, SERVER_PACKET_ID } from "@openao/protocol";

const WS_URL = process.env.GAME_WS_URL || "ws://localhost:7666";

describe("Movement", () => {
  let ws: WebSocket;

  afterAll(() => {
    ws?.close();
  });

  it("should accept position packets without error", async () => {
    ws = new WebSocket(WS_URL);
    ws.binaryType = "arraybuffer";

    await new Promise<void>((resolve) => ws.on("open", resolve));

    for (let heading = 1; heading <= 4; heading++) {
      const writer = new PacketWriter(SERVER_PACKET_ID.position);
      writer.writeByte(heading);
      writer.writeShort(heading);
      ws.send(writer.toArrayBuffer());
    }

    await new Promise<void>((resolve) => setTimeout(resolve, 200));
    expect(ws.readyState).toBe(WebSocket.OPEN);
  });
});
