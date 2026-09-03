import {
  PacketWriter,
  PacketReader,
  SERVER_PACKET_ID,
  CLIENT_PACKET_ID,
  ELR2_SUBPROTOCOL,
  ELR2_ROUTE_AUTHENTICATE,
  ELR2_ROUTE_GAME,
  ELR2_ROUTE_HEARTBEAT,
  FrameKind,
  encodeAuthRequest,
  encodeGameRequest,
  encodeELR2Frame,
  decodeELR2Frame,
  isELR2Frame,
} from "@openao/protocol";

import { TickSynchronizer } from "../network/tickSync";

export type ConnectionState = "disconnected" | "connecting" | "connected" | "authenticating" | "authenticated" | "reconnecting" | "error";

const MAX_RECONNECT_ATTEMPTS = 3;
const RECONNECT_BASE_DELAY_MS = 1000;

class GameSession {
  ws: WebSocket | null = $state(null);
  connectionState: ConnectionState = $state("disconnected");
  error: string = $state("");
  characterId: string = $state("");

  private readonly handlers: Map<number, (reader: PacketReader) => void> = new Map();
  private useELR2 = false;
  private pendingTicket = "";
  private pendingTypeGame = 0;
  private pendingCharIndex = 0;
  private reconnectToken = "";
  private lastWsUrl = "";
  private reconnectAttempts = 0;
  private intentionalDisconnect = false;
  readonly tickSync = new TickSynchronizer();
  private tickSyncProbeInterval: ReturnType<typeof setInterval> | null = null;
  private tickSyncPendingProbes: Map<number, { sentAt: number; localTick: number }> = new Map();
  private tickSyncNextSequence = 1;

  onPacket(packetId: number, handler: (reader: PacketReader) => void) {
    this.handlers.set(packetId, handler);
  }

  connect(wsUrl: string, ticket: string, typeGame: number, charIndex: number) {
    this.disconnect();
    this.connectionState = "connecting";
    this.error = "";
    this.useELR2 = false;
    this.pendingTicket = ticket;
    this.pendingTypeGame = typeGame;
    this.pendingCharIndex = charIndex;
    this.lastWsUrl = wsUrl;
    this.reconnectAttempts = 0;
    this.reconnectToken = "";
    this.intentionalDisconnect = false;

    this.openWebSocket(wsUrl, { ticket: this.pendingTicket });
  }

  disconnect() {
    this.intentionalDisconnect = true;
    this.reconnectToken = "";
    this.reconnectAttempts = 0;
    this.stopTickSyncProbes();
    this.ws?.close();
    this.ws = null;
    this.connectionState = "disconnected";
    this.useELR2 = false;
  }

  private startTickSyncProbes() {
    this.stopTickSyncProbes();
    this.tickSync.reset();
    this.tickSyncPendingProbes.clear();
    this.tickSyncNextSequence = 1;
    this.tickSyncProbeInterval = setInterval(() => this.sendTickSyncProbe(), 5000);
    this.sendTickSyncProbe();
  }

  private stopTickSyncProbes() {
    if (this.tickSyncProbeInterval) {
      clearInterval(this.tickSyncProbeInterval);
      this.tickSyncProbeInterval = null;
    }
  }

  private sendTickSyncProbe() {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN || !this.useELR2) return;
    const seq = this.tickSyncNextSequence++;
    this.tickSyncPendingProbes.set(seq, {
      sentAt: performance.now(),
      localTick: this.tickSync.localTick,
    });
    const frame = encodeELR2Frame(
      FrameKind.Request,
      ELR2_ROUTE_HEARTBEAT,
      new ArrayBuffer(0),
      0,
      seq,
      seq,
    );
    this.ws.send(frame);
  }

  private openWebSocket(wsUrl: string, auth: { ticket: string } | { reconnect_token: string }) {
    const ws = new WebSocket(wsUrl, [ELR2_SUBPROTOCOL]);
    ws.binaryType = "arraybuffer";

    ws.onopen = () => {
      const negotiatedProtocol = ws.protocol;
      if (negotiatedProtocol === ELR2_SUBPROTOCOL) {
        this.useELR2 = true;
        this.connectionState = "authenticating";
        const authPayload = JSON.stringify(auth);
        const frame = encodeAuthRequest(authPayload);
        ws.send(frame);
      } else {
        this.useELR2 = false;
        this.connectionState = "connected";
        this.sendLegacyConnect();
      }
    };

    ws.onmessage = (event) => {
      const data = event.data as ArrayBuffer;

      if (this.useELR2) {
        this.handleELR2Message(data);
      } else {
        this.handleLegacyMessage(data);
      }
    };

    ws.onclose = () => {
      this.ws = null;
      if (this.intentionalDisconnect) {
        this.connectionState = "disconnected";
        return;
      }
      if (this.reconnectToken && this.reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
        this.attemptReconnect();
      } else {
        this.connectionState = "disconnected";
      }
    };

    ws.onerror = () => {
      if (!this.intentionalDisconnect && this.reconnectToken && this.reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
        return;
      }
      this.connectionState = "error";
      this.error = "Error de conexión con el servidor de juego";
    };

    this.ws = ws;
  }

  private attemptReconnect() {
    this.reconnectAttempts++;
    this.connectionState = "reconnecting";
    const delay = RECONNECT_BASE_DELAY_MS * Math.pow(2, this.reconnectAttempts - 1);
    const token = this.reconnectToken;

    setTimeout(() => {
      if (this.intentionalDisconnect || this.connectionState !== "reconnecting") return;
      this.openWebSocket(this.lastWsUrl, { reconnect_token: token });
    }, delay);
  }

  send(data: ArrayBuffer) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      if (this.useELR2) {
        const frame = encodeGameRequest(new Uint8Array(data));
        this.ws.send(frame);
      } else {
        this.ws.send(data);
      }
    }
  }

  sendPing(token: number) {
    const writer = new PacketWriter(SERVER_PACKET_ID.ping);
    writer.writeInt(token);
    this.send(writer.toArrayBuffer());
  }

  private sendLegacyConnect() {
    if (!this.ws) return;
    const writer = new PacketWriter(SERVER_PACKET_ID.connectCharacter);
    writer.writeString(this.pendingTicket);
    writer.writeByte(this.pendingTypeGame);
    writer.writeShort(this.pendingCharIndex);
    this.ws.send(writer.toArrayBuffer());
  }

  private handleELR2Message(data: ArrayBuffer) {
    const frame = decodeELR2Frame(data);
    if (!frame) {
      return;
    }

    if (this.connectionState === "authenticating") {
      if (frame.kind === FrameKind.Response && frame.route === ELR2_ROUTE_AUTHENTICATE) {
        try {
          const decoder = new TextDecoder();
          const json = JSON.parse(decoder.decode(frame.payload));
          if (json.reconnect_token) {
            this.reconnectToken = json.reconnect_token;
          }
        } catch { /* ignore parse errors */ }
        this.connectionState = "authenticated";
        this.reconnectAttempts = 0;
        this.startTickSyncProbes();
        this.sendELR2ConnectCharacter();
      } else if (frame.kind === FrameKind.Error) {
        const decoder = new TextDecoder();
        const errorMsg = decoder.decode(frame.payload);
        if (this.connectionState === "authenticating" && this.reconnectAttempts > 0) {
          this.reconnectToken = "";
          this.connectionState = "disconnected";
        } else {
          this.connectionState = "error";
          this.error = `Auth failed: ${errorMsg}`;
        }
      }
      return;
    }

    if (frame.route === ELR2_ROUTE_AUTHENTICATE && frame.kind === FrameKind.Push) {
      try {
        const decoder = new TextDecoder();
        const json = JSON.parse(decoder.decode(frame.payload));
        if (json.reconnect_token) {
          this.reconnectToken = json.reconnect_token;
        }
      } catch { /* ignore parse errors */ }
      return;
    }

    if (frame.route === ELR2_ROUTE_HEARTBEAT) {
      if (frame.kind === FrameKind.Request) {
        const hbResponse = encodeELR2Frame(
          FrameKind.Response,
          ELR2_ROUTE_HEARTBEAT,
          new ArrayBuffer(0),
          frame.requestIdHigh,
          frame.requestIdLow,
          frame.sequence,
        );
        this.ws?.send(hbResponse);
      } else if (frame.kind === FrameKind.Response && frame.payload.byteLength > 0) {
        try {
          const decoder = new TextDecoder();
          const json = JSON.parse(decoder.decode(frame.payload));
          if (typeof json.server_tick === "number") {
            const probe = this.tickSyncPendingProbes.get(frame.sequence);
            if (probe) {
              this.tickSyncPendingProbes.delete(frame.sequence);
              this.tickSync.observe({
                localTick: probe.localTick,
                serverTick: json.server_tick,
                sentAt: probe.sentAt,
                receivedAt: performance.now(),
                serverReceivedAt: typeof json.server_received_at === "number" ? json.server_received_at : undefined,
                serverSentAt: typeof json.server_sent_at === "number" ? json.server_sent_at : undefined,
              });
            }
          }
        } catch { /* ignore parse errors */ }
      }
      return;
    }

    if (frame.route === ELR2_ROUTE_GAME) {
      const reader = new PacketReader(frame.payload);
      const packetId = reader.getByte();
      if (packetId === CLIENT_PACKET_ID.batch) {
        this.processBatch(reader);
      } else {
        this.dispatchPacket(packetId, reader);
      }
      return;
    }
  }

  private sendELR2ConnectCharacter() {
    const writer = new PacketWriter(SERVER_PACKET_ID.connectCharacter);
    writer.writeString(this.pendingTicket);
    writer.writeByte(this.pendingTypeGame);
    writer.writeShort(this.pendingCharIndex);
    this.send(writer.toArrayBuffer());
  }

  private handleLegacyMessage(data: ArrayBuffer) {
    const reader = new PacketReader(data);
    const packetId = reader.getByte();

    if (packetId === CLIENT_PACKET_ID.batch) {
      this.processBatch(reader);
    } else {
      this.dispatchPacket(packetId, reader);
    }
  }

  private processBatch(reader: PacketReader) {
    while (reader.remainingBytes > 0) {
      const packetId = reader.getByte();
      this.dispatchPacket(packetId, reader);
    }
  }

  private dispatchPacket(packetId: number, reader: PacketReader) {
    const handler = this.handlers.get(packetId);
    if (handler) {
      handler(reader);
    }
  }
}

export const gameSession = new GameSession();
