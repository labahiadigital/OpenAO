import { describe, expect, it } from "vitest";
import {
    CLIENT_PACKET_ID,
    SERVER_PACKET_ID,
    PacketReader,
    PacketWriter,
    decodeClientPacket,
    encodeClientPacket,
    type ClientPacketPayloads,
    type ServerPacketName,
} from "./index.js";

const fixtures = {
    changeHeading: { heading: 3 },
    click: { x: 42, y: 67, button: 1 },
    useItemClick: { slot: 18 },
    equiparItem: { slot: 7 },
    connectCharacter: { ticket: "sesión-🦊", typeGame: 1, idChar: 2 },
    position: { heading: 4, moveId: 4_294_000_000 },
    dialog: { message: "¡Hola, Argentum!" },
    ping: { token: 123_456 },
    attackMele: undefined,
    attackRange: { x: 55, y: 44 },
    attackSpell: { spellSlot: 8, x: 51, y: 49, preferSelfIfEmpty: true },
    tirarItem: { slot: 12, amount: 500 },
    agarrarItem: undefined,
    buyItem: { slot: 4, amount: 25 },
    sellItem: { slot: 9, amount: 13 },
    resyncPosition: undefined,
    changeSeguro: undefined,
    reorderSpell: { sourceSlot: 1, targetSlot: 10 },
    reorderInventoryItem: { sourceSlot: 2, targetSlot: 11 },
    toggleHiddenSkill: undefined,
    useItemU: { slot: 20 },
    changeClanSeguro: undefined,
    craftItem: { profession: "tailoring", itemId: 402, amount: 6 },
    reorderBankItem: { sourceSlot: 3, targetSlot: 12 },
    changeBankTab: { tab: "clan" },
    depositBankGold: { amount: 2_000_000 },
    withdrawBankGold: { amount: 750_000 },
    closeTrade: undefined,
    marketAction: { action: "buy", listingId: "listing-7", expectedPrice: 350 },
    retosAction: { action: "join", challengeId: "challenge-2" },
} satisfies { [K in ServerPacketName]: ClientPacketPayloads[K] };

describe("packet opcodes", () => {
    it("keeps every opcode unique within its direction", () => {
        const clientIds = Object.values(CLIENT_PACKET_ID);
        const serverIds = Object.values(SERVER_PACKET_ID);

        expect(new Set(clientIds).size).toBe(clientIds.length);
        expect(new Set(serverIds).size).toBe(serverIds.length);
    });

    it("has a round-trip fixture for every client-to-server packet", () => {
        expect(Object.keys(fixtures).sort()).toEqual(Object.keys(SERVER_PACKET_ID).sort());
    });
});

describe("client-to-server packet round trips", () => {
    const encode = encodeClientPacket as (type: ServerPacketName, payload?: unknown) => Uint8Array;

    for (const type of Object.keys(fixtures) as ServerPacketName[]) {
        it(`round-trips ${type}`, () => {
            const payload = fixtures[type];
            const encoded = encode(type, payload);
            const decoded = decodeClientPacket(encoded);

            expect(encoded).toMatchSnapshot();
            expect(decoded.type).toBe(type);
            expect(decoded.id).toBe(SERVER_PACKET_ID[type]);
            expect(decoded.payload).toEqual(payload);
        });
    }
});

describe("binary primitives", () => {
    it("round-trips every primitive and preserves Unicode character lengths", () => {
        const writer = new PacketWriter();
        writer.writeByte(255);
        writer.writeByte(-12, true);
        writer.writeShort(65_535);
        writer.writeShort(-12_345, true);
        writer.writeInt(4_294_967_295);
        writer.writeInt(-123_456_789, true);
        writer.writeFloat(123.5);
        writer.writeDouble(Math.PI);
        writer.writeString("áéí 🦊");

        const reader = new PacketReader(writer.toUint8Array());
        expect(reader.getByte()).toBe(255);
        expect(reader.getByte(true)).toBe(-12);
        expect(reader.getShort()).toBe(65_535);
        expect(reader.getShort(true)).toBe(-12_345);
        expect(reader.getInt()).toBe(4_294_967_295);
        expect(reader.getInt(true)).toBe(-123_456_789);
        expect(reader.getFloat()).toBe(123.5);
        expect(reader.getDouble()).toBe(Math.PI);
        expect(reader.getString()).toBe("áéí 🦊");
        expect(reader.remainingBytes).toBe(0);
    });

    it("rejects unknown, truncated, and trailing packet bytes", () => {
        expect(() => decodeClientPacket(Uint8Array.of(255))).toThrow(/Unknown/);
        expect(() => decodeClientPacket(Uint8Array.of(SERVER_PACKET_ID.position, 1))).toThrow(/Cannot read/);

        const valid = encodeClientPacket("changeHeading", { heading: 2 });
        expect(() => decodeClientPacket(Uint8Array.from([...valid, 99]))).toThrow(/trailing bytes/);
    });
});

/* ───── ELR2 Framing Tests ───── */

import {
    ELR2_MAGIC,
    ELR2_VERSION,
    ELR2_HEADER_LEN,
    ELR2_SUBPROTOCOL,
    ELR2_ROUTE_AUTHENTICATE,
    ELR2_ROUTE_HEARTBEAT,
    ELR2_ROUTE_GAME,
    FrameKind,
    encodeELR2Frame,
    encodeAuthRequest,
    encodeGameRequest,
    decodeELR2Frame,
    isELR2Frame,
} from "./elr2.js";

describe("ELR2 constants", () => {
    it("magic bytes spell 'ELR2' in ASCII", () => {
        expect(ELR2_MAGIC).toBe(0x454C5232);
        const buf = new ArrayBuffer(4);
        new DataView(buf).setUint32(0, ELR2_MAGIC, false);
        expect(new TextDecoder().decode(buf)).toBe("ELR2");
    });

    it("has correct version and header length", () => {
        expect(ELR2_VERSION).toBe(2);
        expect(ELR2_HEADER_LEN).toBe(28);
    });

    it("has correct subprotocol string", () => {
        expect(ELR2_SUBPROTOCOL).toBe("elura.v2");
    });

    it("has correct route constants", () => {
        expect(ELR2_ROUTE_AUTHENTICATE).toBe(1);
        expect(ELR2_ROUTE_HEARTBEAT).toBe(2);
        expect(ELR2_ROUTE_GAME).toBe(100);
    });
});

describe("ELR2 encode/decode roundtrip", () => {
    it("encodes and decodes a request frame", () => {
        const payload = new TextEncoder().encode("test payload");
        const encoded = encodeELR2Frame(FrameKind.Request, 100, payload, 0, 42, 7);

        expect(encoded.byteLength).toBe(ELR2_HEADER_LEN + payload.byteLength);

        const frame = decodeELR2Frame(encoded);
        expect(frame).not.toBeNull();
        expect(frame!.kind).toBe(FrameKind.Request);
        expect(frame!.route).toBe(100);
        expect(frame!.requestIdLow).toBe(42);
        expect(frame!.sequence).toBe(7);
        expect(new TextDecoder().decode(frame!.payload)).toBe("test payload");
    });

    it("encodes and decodes a push frame with empty payload", () => {
        const encoded = encodeELR2Frame(FrameKind.Push, ELR2_ROUTE_HEARTBEAT, new ArrayBuffer(0));
        expect(encoded.byteLength).toBe(ELR2_HEADER_LEN);

        const frame = decodeELR2Frame(encoded);
        expect(frame).not.toBeNull();
        expect(frame!.kind).toBe(FrameKind.Push);
        expect(frame!.route).toBe(ELR2_ROUTE_HEARTBEAT);
        expect(frame!.payload.byteLength).toBe(0);
    });

    it("encodes and decodes a response frame", () => {
        const payload = new TextEncoder().encode('{"status":"ok"}');
        const encoded = encodeELR2Frame(FrameKind.Response, 1, payload, 0, 1, 0);

        const frame = decodeELR2Frame(encoded);
        expect(frame).not.toBeNull();
        expect(frame!.kind).toBe(FrameKind.Response);
        expect(frame!.route).toBe(1);
    });

    it("encodes and decodes an error frame", () => {
        const payload = new TextEncoder().encode("invalid ticket");
        const encoded = encodeELR2Frame(FrameKind.Error, 1, payload, 0, 1, 0);

        const frame = decodeELR2Frame(encoded);
        expect(frame).not.toBeNull();
        expect(frame!.kind).toBe(FrameKind.Error);
        expect(new TextDecoder().decode(frame!.payload)).toBe("invalid ticket");
    });
});

describe("ELR2 helper functions", () => {
    it("encodeAuthRequest creates a Route 1 request", () => {
        const encoded = encodeAuthRequest('{"ticket":"abc123"}');
        const frame = decodeELR2Frame(encoded);
        expect(frame).not.toBeNull();
        expect(frame!.kind).toBe(FrameKind.Request);
        expect(frame!.route).toBe(ELR2_ROUTE_AUTHENTICATE);
        expect(new TextDecoder().decode(frame!.payload)).toBe('{"ticket":"abc123"}');
    });

    it("encodeGameRequest creates a Route 100 request", () => {
        const gamePayload = new Uint8Array([1, 0, 42]);
        const encoded = encodeGameRequest(gamePayload);
        const frame = decodeELR2Frame(encoded);
        expect(frame).not.toBeNull();
        expect(frame!.kind).toBe(FrameKind.Request);
        expect(frame!.route).toBe(ELR2_ROUTE_GAME);
        const payloadBytes = new Uint8Array(frame!.payload);
        expect(payloadBytes[0]).toBe(1);
        expect(payloadBytes[1]).toBe(0);
        expect(payloadBytes[2]).toBe(42);
    });
});

describe("isELR2Frame detection", () => {
    it("detects valid ELR2 frame", () => {
        const frame = encodeELR2Frame(FrameKind.Push, 100, new ArrayBuffer(0));
        expect(isELR2Frame(frame)).toBe(true);
    });

    it("rejects non-ELR2 data", () => {
        const raw = new Uint8Array([1, 2, 3, 4, 5]).buffer;
        expect(isELR2Frame(raw)).toBe(false);
    });

    it("rejects data shorter than 4 bytes", () => {
        expect(isELR2Frame(new ArrayBuffer(3))).toBe(false);
    });
});

describe("ELR2 decode error handling", () => {
    it("returns null for data shorter than header", () => {
        expect(decodeELR2Frame(new ArrayBuffer(20))).toBeNull();
    });

    it("returns null for invalid magic", () => {
        const buf = new ArrayBuffer(ELR2_HEADER_LEN);
        const view = new DataView(buf);
        view.setUint32(0, 0xDEADBEEF, false);
        expect(decodeELR2Frame(buf)).toBeNull();
    });

    it("returns null for wrong version", () => {
        const buf = new ArrayBuffer(ELR2_HEADER_LEN);
        const view = new DataView(buf);
        view.setUint32(0, ELR2_MAGIC, false);
        view.setUint16(4, 99, false);
        expect(decodeELR2Frame(buf)).toBeNull();
    });

    it("returns null for truncated payload", () => {
        const buf = new ArrayBuffer(ELR2_HEADER_LEN);
        const view = new DataView(buf);
        view.setUint32(0, ELR2_MAGIC, false);
        view.setUint16(4, ELR2_VERSION, false);
        view.setUint8(6, FrameKind.Request);
        view.setUint32(8, 100, false);
        view.setUint32(24, 999, false); // claims 999 bytes
        expect(decodeELR2Frame(buf)).toBeNull();
    });
});
