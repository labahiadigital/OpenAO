/**
 * ELR2 protocol framing for OpenAO.
 *
 * Header layout (28 bytes, big-endian):
 *   [0..4]   u32  magic   = 0x454C5232 ("ELR2")
 *   [4..6]   u16  version = 2
 *   [6]      u8   kind    (1=Request, 2=Response, 3=Push, 4=Error)
 *   [7]      u8   flags
 *   [8..12]  u32  route
 *   [12..20] u64  request_id (as two u32: high + low)
 *   [20..24] u32  sequence
 *   [24..28] u32  payload_length
 *   [28..]   payload bytes
 */

export const ELR2_MAGIC = 0x454c5232;
export const ELR2_VERSION = 2;
export const ELR2_HEADER_LEN = 28;
export const ELR2_SUBPROTOCOL = "elura.v2";

export const ELR2_ROUTE_AUTHENTICATE = 1;
export const ELR2_ROUTE_HEARTBEAT = 2;
export const ELR2_ROUTE_GAME = 100;

export const enum FrameKind {
    Request = 1,
    Response = 2,
    Push = 3,
    Error = 4,
}

export interface ELR2Frame {
    kind: FrameKind;
    flags: number;
    route: number;
    requestIdHigh: number;
    requestIdLow: number;
    sequence: number;
    payload: ArrayBuffer;
}

let nextRequestId = 1;

export function encodeELR2Frame(
    kind: FrameKind,
    route: number,
    payload: ArrayBuffer | Uint8Array,
    requestIdHigh = 0,
    requestIdLow = 0,
    sequence = 0,
    flags = 0,
): ArrayBuffer {
    const payloadBytes = payload instanceof Uint8Array ? payload : new Uint8Array(payload);
    const totalLen = ELR2_HEADER_LEN + payloadBytes.byteLength;
    const buffer = new ArrayBuffer(totalLen);
    const view = new DataView(buffer);

    view.setUint32(0, ELR2_MAGIC, false);
    view.setUint16(4, ELR2_VERSION, false);
    view.setUint8(6, kind);
    view.setUint8(7, flags);
    view.setUint32(8, route, false);
    view.setUint32(12, requestIdHigh, false);
    view.setUint32(16, requestIdLow, false);
    view.setUint32(20, sequence, false);
    view.setUint32(24, payloadBytes.byteLength, false);

    const output = new Uint8Array(buffer);
    output.set(payloadBytes, ELR2_HEADER_LEN);

    return buffer;
}

export function encodeRequest(route: number, payload: ArrayBuffer | Uint8Array): ArrayBuffer {
    const id = nextRequestId++;
    return encodeELR2Frame(FrameKind.Request, route, payload, 0, id);
}

export function encodeAuthRequest(ticketJson: string): ArrayBuffer {
    const encoder = new TextEncoder();
    const payload = encoder.encode(ticketJson);
    return encodeRequest(ELR2_ROUTE_AUTHENTICATE, payload);
}

export function encodeGameRequest(gamePayload: ArrayBuffer | Uint8Array): ArrayBuffer {
    return encodeRequest(ELR2_ROUTE_GAME, gamePayload);
}

export function decodeELR2Frame(data: ArrayBuffer): ELR2Frame | null {
    if (data.byteLength < ELR2_HEADER_LEN) {
        return null;
    }

    const view = new DataView(data);
    const magic = view.getUint32(0, false);
    if (magic !== ELR2_MAGIC) {
        return null;
    }

    const version = view.getUint16(4, false);
    if (version !== ELR2_VERSION) {
        return null;
    }

    const kind = view.getUint8(6) as FrameKind;
    const flags = view.getUint8(7);
    const route = view.getUint32(8, false);
    const requestIdHigh = view.getUint32(12, false);
    const requestIdLow = view.getUint32(16, false);
    const sequence = view.getUint32(20, false);
    const payloadLength = view.getUint32(24, false);

    if (data.byteLength < ELR2_HEADER_LEN + payloadLength) {
        return null;
    }

    const payload = data.slice(ELR2_HEADER_LEN, ELR2_HEADER_LEN + payloadLength);

    return {
        kind,
        flags,
        route,
        requestIdHigh,
        requestIdLow,
        sequence,
        payload,
    };
}

export function isELR2Frame(data: ArrayBuffer): boolean {
    if (data.byteLength < 4) return false;
    const view = new DataView(data);
    return view.getUint32(0, false) === ELR2_MAGIC;
}
