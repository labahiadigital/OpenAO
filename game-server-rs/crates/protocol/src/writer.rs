/// Binary packet writer compatible with the TypeScript `PacketWriter`.
///
/// All multi-byte integers are little-endian to match the JS `DataView`
/// calls in `binary.ts`.
pub struct PacketWriter {
    buf: Vec<u8>,
}

impl PacketWriter {
    pub fn new() -> Self {
        Self { buf: Vec::with_capacity(128) }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self { buf: Vec::with_capacity(cap) }
    }

    pub fn with_packet_id(packet_id: u8) -> Self {
        let mut w = Self::new();
        w.write_byte(packet_id);
        w
    }

    pub fn with_packet_id_and_capacity(packet_id: u8, cap: usize) -> Self {
        let mut w = Self::with_capacity(cap);
        w.write_byte(packet_id);
        w
    }

    pub fn write_byte(&mut self, value: u8) {
        self.buf.push(value);
    }

    pub fn write_byte_signed(&mut self, value: i8) {
        self.buf.push(value as u8);
    }

    pub fn write_short(&mut self, value: u16) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_short_signed(&mut self, value: i16) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_int(&mut self, value: u32) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_int_signed(&mut self, value: i32) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_float(&mut self, value: f32) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_double(&mut self, value: f64) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    /// Writes a UTF-8 string prefixed by its **character count** as a u16
    /// (matching the TS implementation which writes `Array.from(str).length`).
    pub fn write_string(&mut self, value: &str) {
        let char_count = value.chars().count() as u16;
        self.write_short(char_count);
        self.buf.extend_from_slice(value.as_bytes());
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

impl Default for PacketWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_byte() {
        let mut w = PacketWriter::new();
        w.write_byte(42);
        assert_eq!(w.into_bytes(), vec![42]);
    }

    #[test]
    fn write_short_le() {
        let mut w = PacketWriter::new();
        w.write_short(0x0102);
        assert_eq!(w.into_bytes(), vec![0x02, 0x01]);
    }

    #[test]
    fn write_int_le() {
        let mut w = PacketWriter::new();
        w.write_int(0x01020304);
        assert_eq!(w.into_bytes(), vec![0x04, 0x03, 0x02, 0x01]);
    }

    #[test]
    fn write_string_ascii() {
        let mut w = PacketWriter::new();
        w.write_string("hi");
        let bytes = w.into_bytes();
        assert_eq!(bytes[0..2], [2, 0]); // char count = 2, LE
        assert_eq!(&bytes[2..], b"hi");
    }

    #[test]
    fn write_string_unicode() {
        let mut w = PacketWriter::new();
        w.write_string("ñ");
        let bytes = w.into_bytes();
        assert_eq!(bytes[0..2], [1, 0]); // 1 character
        assert_eq!(&bytes[2..], "ñ".as_bytes()); // 2 UTF-8 bytes
    }

    #[test]
    fn write_float_le() {
        let mut w = PacketWriter::new();
        w.write_float(1.0);
        assert_eq!(w.into_bytes(), 1.0_f32.to_le_bytes().to_vec());
    }

    #[test]
    fn write_double_le() {
        let mut w = PacketWriter::new();
        w.write_double(1.0);
        assert_eq!(w.into_bytes(), 1.0_f64.to_le_bytes().to_vec());
    }
}
