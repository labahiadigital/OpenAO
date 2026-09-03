use std::string::FromUtf8Error;

/// Error returned when reading past the end of the packet buffer or when
/// encountering invalid data.
#[derive(Debug)]
pub enum ReadError {
    OutOfBounds { requested: usize, remaining: usize },
    InvalidUtf8(FromUtf8Error),
    InvalidUtf8LeadingByte(u8),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds { requested, remaining } => {
                write!(f, "Cannot read {requested} bytes; {remaining} remain")
            }
            Self::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {e}"),
            Self::InvalidUtf8LeadingByte(b) => {
                write!(f, "Invalid UTF-8 leading byte: {b}")
            }
        }
    }
}

impl std::error::Error for ReadError {}

/// Binary packet reader compatible with the TypeScript `PacketReader`.
///
/// All multi-byte integers are little-endian.
pub struct PacketReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> PacketReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.offset
    }

    pub fn can_read(&self, len: usize) -> bool {
        self.offset + len <= self.data.len()
    }

    fn assert_readable(&self, len: usize) -> Result<(), ReadError> {
        if self.can_read(len) {
            Ok(())
        } else {
            Err(ReadError::OutOfBounds {
                requested: len,
                remaining: self.remaining(),
            })
        }
    }

    pub fn get_byte(&mut self) -> Result<u8, ReadError> {
        self.assert_readable(1)?;
        let v = self.data[self.offset];
        self.offset += 1;
        Ok(v)
    }

    pub fn get_byte_signed(&mut self) -> Result<i8, ReadError> {
        Ok(self.get_byte()? as i8)
    }

    pub fn get_short(&mut self) -> Result<u16, ReadError> {
        self.assert_readable(2)?;
        let v = u16::from_le_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
        ]);
        self.offset += 2;
        Ok(v)
    }

    pub fn get_short_signed(&mut self) -> Result<i16, ReadError> {
        self.assert_readable(2)?;
        let v = i16::from_le_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
        ]);
        self.offset += 2;
        Ok(v)
    }

    pub fn get_int(&mut self) -> Result<u32, ReadError> {
        self.assert_readable(4)?;
        let v = u32::from_le_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ]);
        self.offset += 4;
        Ok(v)
    }

    pub fn get_int_signed(&mut self) -> Result<i32, ReadError> {
        self.assert_readable(4)?;
        let v = i32::from_le_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ]);
        self.offset += 4;
        Ok(v)
    }

    pub fn get_float(&mut self) -> Result<f32, ReadError> {
        self.assert_readable(4)?;
        let v = f32::from_le_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ]);
        self.offset += 4;
        Ok(v)
    }

    pub fn get_double(&mut self) -> Result<f64, ReadError> {
        self.assert_readable(8)?;
        let bytes: [u8; 8] = self.data[self.offset..self.offset + 8]
            .try_into()
            .unwrap();
        let v = f64::from_le_bytes(bytes);
        self.offset += 8;
        Ok(v)
    }

    /// Reads a UTF-8 string with the same encoding as the TS implementation:
    /// a u16 **character count** followed by the UTF-8 bytes.
    pub fn get_string(&mut self) -> Result<String, ReadError> {
        let char_count = self.get_short()? as usize;
        let byte_len = self.utf8_byte_length(char_count)?;
        self.assert_readable(byte_len)?;
        let bytes = &self.data[self.offset..self.offset + byte_len];
        self.offset += byte_len;
        String::from_utf8(bytes.to_vec()).map_err(ReadError::InvalidUtf8)
    }

    pub fn get_bytes(&mut self, len: usize) -> Result<&'a [u8], ReadError> {
        self.assert_readable(len)?;
        let slice = &self.data[self.offset..self.offset + len];
        self.offset += len;
        Ok(slice)
    }

    fn utf8_byte_length(&self, char_count: usize) -> Result<usize, ReadError> {
        let mut byte_len = 0usize;

        for _ in 0..char_count {
            if self.offset + byte_len >= self.data.len() {
                return Err(ReadError::OutOfBounds {
                    requested: byte_len + 1,
                    remaining: self.data.len() - self.offset,
                });
            }

            let b = self.data[self.offset + byte_len];
            if b & 0x80 == 0 {
                byte_len += 1;
            } else if b & 0xE0 == 0xC0 {
                byte_len += 2;
            } else if b & 0xF0 == 0xE0 {
                byte_len += 3;
            } else if b & 0xF8 == 0xF0 {
                byte_len += 4;
            } else {
                return Err(ReadError::InvalidUtf8LeadingByte(b));
            }
        }

        Ok(byte_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::PacketWriter;

    #[test]
    fn round_trip_byte() {
        let mut w = PacketWriter::new();
        w.write_byte(255);
        let mut r = PacketReader::new(w.as_bytes());
        assert_eq!(r.get_byte().unwrap(), 255);
    }

    #[test]
    fn round_trip_short() {
        let mut w = PacketWriter::new();
        w.write_short(12345);
        let mut r = PacketReader::new(w.as_bytes());
        assert_eq!(r.get_short().unwrap(), 12345);
    }

    #[test]
    fn round_trip_int() {
        let mut w = PacketWriter::new();
        w.write_int(987654321);
        let mut r = PacketReader::new(w.as_bytes());
        assert_eq!(r.get_int().unwrap(), 987654321);
    }

    #[test]
    fn round_trip_float() {
        let mut w = PacketWriter::new();
        w.write_float(3.14);
        let mut r = PacketReader::new(w.as_bytes());
        let v = r.get_float().unwrap();
        assert!((v - 3.14).abs() < 0.001);
    }

    #[test]
    fn round_trip_string_ascii() {
        let mut w = PacketWriter::new();
        w.write_string("hello world");
        let mut r = PacketReader::new(w.as_bytes());
        assert_eq!(r.get_string().unwrap(), "hello world");
    }

    #[test]
    fn round_trip_string_unicode() {
        let mut w = PacketWriter::new();
        w.write_string("ñoño 😀");
        let mut r = PacketReader::new(w.as_bytes());
        assert_eq!(r.get_string().unwrap(), "ñoño 😀");
    }

    #[test]
    fn out_of_bounds() {
        let mut r = PacketReader::new(&[1]);
        assert!(r.get_short().is_err());
    }

    #[test]
    fn complex_packet() {
        let mut w = PacketWriter::with_packet_id(42);
        w.write_short(100);
        w.write_int(999);
        w.write_string("test");
        w.write_byte(0);

        let mut r = PacketReader::new(w.as_bytes());
        assert_eq!(r.get_byte().unwrap(), 42);
        assert_eq!(r.get_short().unwrap(), 100);
        assert_eq!(r.get_int().unwrap(), 999);
        assert_eq!(r.get_string().unwrap(), "test");
        assert_eq!(r.get_byte().unwrap(), 0);
        assert_eq!(r.remaining(), 0);
    }
}
