import { describe, it, expect, beforeAll, afterAll } from "vitest";
import WebSocket from "ws";
import { PacketWriter, PacketReader, SERVER_PACKET_ID, CLIENT_PACKET_ID } from "@openao/protocol";

const WS_URL = process.env.GAME_WS_URL || "ws://localhost:7666";

describe("Game Server Connection", () => {
  let ws: WebSocket;

  afterAll(() => {
    ws?.close();
  });

  it("should connect via WebSocket", async () => {
    ws = new WebSocket(WS_URL);

    await new Promise<void>((resolve, reject) => {
      ws.on("open", resolve);
      ws.on("error", reject);
      setTimeout(() => reject(new Error("Connection timeout")), 5000);
    });

    expect(ws.readyState).toBe(WebSocket.OPEN);
  });

  it("should respond to ping with pong", async () => {
    ws = new WebSocket(WS_URL);
    ws.binaryType = "arraybuffer";

    await new Promise<void>((resolve) => ws.on("open", resolve));

    const token = 42;
    const writer = new PacketWriter(SERVER_PACKET_ID.ping);
    writer.writeInt(token);
    ws.send(writer.toArrayBuffer());

    const response = await new Promise<ArrayBuffer>((resolve, reject) => {
      ws.on("message", (data) => resolve(data as unknown as ArrayBuffer));
      setTimeout(() => reject(new Error("Pong timeout")), 5000);
    });

    const reader = new PacketReader(response);
    const packetId = reader.getByte();
    const responseToken = reader.getInt();

    expect(packetId).toBe(CLIENT_PACKET_ID.pong);
    expect(responseToken).toBe(token);
  });

  it("should handle connectCharacter packet", async () => {
    ws = new WebSocket(WS_URL);
    ws.binaryType = "arraybuffer";

    await new Promise<void>((resolve) => ws.on("open", resolve));

    const writer = new PacketWriter(SERVER_PACKET_ID.connectCharacter);
    writer.writeString("test-ticket-invalid");
    writer.writeByte(0);
    writer.writeShort(0);
    ws.send(writer.toArrayBuffer());

    // Server should handle gracefully (no crash)
    await new Promise<void>((resolve) => setTimeout(resolve, 500));
    expect(ws.readyState).toBe(WebSocket.OPEN);
  });
});
