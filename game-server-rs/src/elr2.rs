use bytes::{Buf, BufMut, Bytes, BytesMut};

pub const ELR2_MAGIC: u32 = 0x454C_5232;
pub const ELR2_VERSION: u16 = 2;
pub const ELR2_HEADER_LEN: usize = 28;
pub const SUBPROTOCOL: &str = "elura.v2";

pub const ROUTE_AUTHENTICATE: u32 = 1;
pub const ROUTE_HEARTBEAT: u32 = 2;

pub const ROUTE_GAME: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameKind {
    Request = 1,
    Response = 2,
    Push = 3,
    Error = 4,
}

impl TryFrom<u8> for FrameKind {
    type Error = &'static str;
    fn try_from(v: u8) -> Result<Self, &'static str> {
        match v {
            1 => Ok(Self::Request),
            2 => Ok(Self::Response),
            3 => Ok(Self::Push),
            4 => Ok(Self::Error),
            _ => Err("unknown frame kind"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub kind: FrameKind,
    pub flags: u8,
    pub route: u32,
    pub request_id: u64,
    pub sequence: u32,
    pub payload: Bytes,
}

impl Frame {
    pub fn response(request: &Frame, payload: impl Into<Bytes>) -> Self {
        Self {
            kind: FrameKind::Response,
            flags: 0,
            route: request.route,
            request_id: request.request_id,
            sequence: request.sequence,
            payload: payload.into(),
        }
    }

    pub fn error_response(request: &Frame, payload: impl Into<Bytes>) -> Self {
        Self {
            kind: FrameKind::Error,
            flags: 0,
            route: request.route,
            request_id: request.request_id,
            sequence: request.sequence,
            payload: payload.into(),
        }
    }

    pub fn push(route: u32, payload: impl Into<Bytes>) -> Self {
        Self {
            kind: FrameKind::Push,
            flags: 0,
            route,
            request_id: 0,
            sequence: 0,
            payload: payload.into(),
        }
    }

    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(ELR2_HEADER_LEN + self.payload.len());
        buf.put_u32(ELR2_MAGIC);
        buf.put_u16(ELR2_VERSION);
        buf.put_u8(self.kind as u8);
        buf.put_u8(self.flags);
        buf.put_u32(self.route);
        buf.put_u64(self.request_id);
        buf.put_u32(self.sequence);
        buf.put_u32(self.payload.len() as u32);
        buf.extend_from_slice(&self.payload);
        buf.freeze()
    }

    pub fn decode(mut data: Bytes) -> Result<Self, &'static str> {
        if data.len() < ELR2_HEADER_LEN {
            return Err("frame too short");
        }
        let magic = data.get_u32();
        if magic != ELR2_MAGIC {
            return Err("invalid ELR2 magic");
        }
        let version = data.get_u16();
        if version != ELR2_VERSION {
            return Err("unsupported ELR2 version");
        }
        let kind = FrameKind::try_from(data.get_u8())?;
        let flags = data.get_u8();
        let route = data.get_u32();
        let request_id = data.get_u64();
        let sequence = data.get_u32();
        let payload_len = data.get_u32() as usize;
        if data.remaining() < payload_len {
            return Err("incomplete payload");
        }
        let payload = data.split_to(payload_len);
        Ok(Self {
            kind,
            flags,
            route,
            request_id,
            sequence,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let original = Frame {
            kind: FrameKind::Request,
            flags: 0,
            route: ROUTE_GAME,
            request_id: 42,
            sequence: 7,
            payload: Bytes::from_static(b"hello"),
        };
        let encoded = original.encode();
        assert_eq!(encoded.len(), ELR2_HEADER_LEN + 5);

        let decoded = Frame::decode(encoded).unwrap();
        assert_eq!(decoded.kind, FrameKind::Request);
        assert_eq!(decoded.flags, 0);
        assert_eq!(decoded.route, ROUTE_GAME);
        assert_eq!(decoded.request_id, 42);
        assert_eq!(decoded.sequence, 7);
        assert_eq!(&decoded.payload[..], b"hello");
    }

    #[test]
    fn push_frame_has_zero_ids() {
        let frame = Frame::push(ROUTE_GAME, Bytes::from_static(b"data"));
        assert_eq!(frame.kind, FrameKind::Push);
        assert_eq!(frame.request_id, 0);
        assert_eq!(frame.sequence, 0);
        assert_eq!(frame.route, ROUTE_GAME);
    }

    #[test]
    fn response_preserves_request_fields() {
        let request = Frame {
            kind: FrameKind::Request,
            flags: 0,
            route: ROUTE_AUTHENTICATE,
            request_id: 99,
            sequence: 3,
            payload: Bytes::new(),
        };
        let response = Frame::response(&request, Bytes::from_static(b"ok"));
        assert_eq!(response.kind, FrameKind::Response);
        assert_eq!(response.route, ROUTE_AUTHENTICATE);
        assert_eq!(response.request_id, 99);
        assert_eq!(response.sequence, 3);
        assert_eq!(&response.payload[..], b"ok");
    }

    #[test]
    fn error_response_has_error_kind() {
        let request = Frame {
            kind: FrameKind::Request,
            flags: 0,
            route: ROUTE_AUTHENTICATE,
            request_id: 1,
            sequence: 0,
            payload: Bytes::new(),
        };
        let err = Frame::error_response(&request, Bytes::from_static(b"fail"));
        assert_eq!(err.kind, FrameKind::Error);
        assert_eq!(&err.payload[..], b"fail");
    }

    #[test]
    fn decode_too_short_fails() {
        let short = Bytes::from_static(&[0u8; 10]);
        assert!(Frame::decode(short).is_err());
    }

    #[test]
    fn decode_bad_magic_fails() {
        let mut buf = BytesMut::with_capacity(ELR2_HEADER_LEN);
        buf.put_u32(0xDEADBEEF);
        buf.put_u16(ELR2_VERSION);
        buf.put_u8(1);
        buf.put_u8(0);
        buf.put_u32(100);
        buf.put_u64(0);
        buf.put_u32(0);
        buf.put_u32(0);
        assert!(Frame::decode(buf.freeze()).is_err());
    }

    #[test]
    fn decode_bad_version_fails() {
        let mut buf = BytesMut::with_capacity(ELR2_HEADER_LEN);
        buf.put_u32(ELR2_MAGIC);
        buf.put_u16(99);
        buf.put_u8(1);
        buf.put_u8(0);
        buf.put_u32(100);
        buf.put_u64(0);
        buf.put_u32(0);
        buf.put_u32(0);
        assert!(Frame::decode(buf.freeze()).is_err());
    }

    #[test]
    fn decode_incomplete_payload_fails() {
        let mut buf = BytesMut::with_capacity(ELR2_HEADER_LEN);
        buf.put_u32(ELR2_MAGIC);
        buf.put_u16(ELR2_VERSION);
        buf.put_u8(1);
        buf.put_u8(0);
        buf.put_u32(100);
        buf.put_u64(0);
        buf.put_u32(0);
        buf.put_u32(100); // claims 100 bytes payload
        // but only has 0 bytes
        assert!(Frame::decode(buf.freeze()).is_err());
    }

    #[test]
    fn empty_payload_roundtrip() {
        let frame = Frame::push(ROUTE_HEARTBEAT, Bytes::new());
        let encoded = frame.encode();
        assert_eq!(encoded.len(), ELR2_HEADER_LEN);

        let decoded = Frame::decode(encoded).unwrap();
        assert_eq!(decoded.route, ROUTE_HEARTBEAT);
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn frame_kind_try_from_valid() {
        assert_eq!(FrameKind::try_from(1u8).unwrap(), FrameKind::Request);
        assert_eq!(FrameKind::try_from(2u8).unwrap(), FrameKind::Response);
        assert_eq!(FrameKind::try_from(3u8).unwrap(), FrameKind::Push);
        assert_eq!(FrameKind::try_from(4u8).unwrap(), FrameKind::Error);
    }

    #[test]
    fn frame_kind_try_from_invalid() {
        assert!(FrameKind::try_from(0u8).is_err());
        assert!(FrameKind::try_from(5u8).is_err());
        assert!(FrameKind::try_from(255u8).is_err());
    }

    #[test]
    fn magic_bytes_are_elr2_ascii() {
        assert_eq!(ELR2_MAGIC, 0x454C5232);
        let bytes = ELR2_MAGIC.to_be_bytes();
        assert_eq!(&bytes, b"ELR2");
    }
}
