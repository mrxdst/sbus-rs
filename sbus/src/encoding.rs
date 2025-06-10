use bytes::Buf;
use std::{
    io::{Cursor, Read},
    num::TryFromIntError,
};

#[derive(PartialEq, Debug)]
pub enum EncodeError {
    Overflow,
}

impl From<TryFromIntError> for EncodeError {
    fn from(_: TryFromIntError) -> Self {
        Self::Overflow
    }
}

pub type EncodeResult = Result<(), EncodeError>;

pub trait Encodable {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult;

    fn encode_to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        Encoder::encode(self)
    }
}

pub struct Encoder {
    buffer: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        return Self {
            buffer: Vec::with_capacity(16),
        };
    }

    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }

    #[allow(unused)]
    pub fn position(&self) -> usize {
        return self.buffer.len();
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.buffer.extend(value.to_be_bytes());
    }

    pub fn write_i32(&mut self, value: i32) {
        self.buffer.extend(value.to_be_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.buffer.extend(value.to_be_bytes());
    }

    pub fn write_string(&mut self, value: &str) {
        self.write_bytes(value.as_bytes());
        self.write_u8(0);
    }

    pub fn write_bools(&mut self, values: &[bool]) {
        let byte_length = (values.len() + 7) / 8;
        self.buffer.reserve(byte_length);
        for i in 0..byte_length {
            let mut byte = 0;
            for i2 in 0..8 {
                if values.get((i * 8 + i2) as usize).copied().unwrap_or_default() {
                    byte |= 1 << i2;
                }
            }
            self.write_u8(byte);
        }
    }

    pub fn write_bytes(&mut self, value: &[u8]) {
        self.buffer.extend(value);
    }

    pub fn write_type<T>(&mut self, value: &T) -> EncodeResult
    where
        T: Encodable + ?Sized,
    {
        return value.encode(self);
    }

    pub fn finish(self) -> Vec<u8> {
        return self.buffer;
    }

    pub fn encode<T>(value: &T) -> Result<Vec<u8>, EncodeError>
    where
        T: Encodable + ?Sized,
    {
        let mut encoder = Self::new();
        encoder.write_type(value)?;
        return Ok(encoder.finish());
    }
}

#[derive(PartialEq, Debug)]
pub enum DecodeError {
    MissingData,
    InvalidData(String),
}

pub type DecodeResult<T> = Result<T, DecodeError>;

pub trait Decodable<T> {
    fn decode(decoder: &mut Decoder) -> DecodeResult<T>;

    fn decode_from_bytes(buffer: &[u8]) -> DecodeResult<T>
    where
        T: Decodable<T>,
    {
        Decoder::decode(buffer)
    }
}

pub struct Decoder<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> Decoder<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(&buffer),
        }
    }

    #[allow(unused)]
    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }

    #[allow(unused)]
    pub fn remaining(&self) -> usize {
        return self.cursor.remaining();
    }

    pub fn read_u8(&mut self) -> DecodeResult<u8> {
        if self.cursor.remaining() < 1 {
            return Err(DecodeError::MissingData);
        }
        Ok(self.cursor.get_u8())
    }

    pub fn read_u16(&mut self) -> DecodeResult<u16> {
        if self.cursor.remaining() < 2 {
            return Err(DecodeError::MissingData);
        }
        Ok(self.cursor.get_u16())
    }

    pub fn read_i32(&mut self) -> DecodeResult<i32> {
        if self.cursor.remaining() < 4 {
            return Err(DecodeError::MissingData);
        }
        Ok(self.cursor.get_i32())
    }

    pub fn read_u32(&mut self) -> DecodeResult<u32> {
        if self.cursor.remaining() < 4 {
            return Err(DecodeError::MissingData);
        }
        Ok(self.cursor.get_u32())
    }

    pub fn read_string(&mut self) -> DecodeResult<String> {
        let mut bytes = Vec::new();

        loop {
            let byte = self.read_u8()?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }

        let text = String::from_utf8_lossy(&bytes).into();

        Ok(text)
    }

    pub fn read_bools(&mut self, length: usize) -> DecodeResult<Vec<bool>> {
        let byte_length = (length + 7) / 8;
        let mut values = Vec::with_capacity(length);
        for _ in 0..byte_length {
            let byte = self.read_u8()?;
            for i2 in 0..8 {
                if values.len() == length {
                    break;
                }
                values.push((byte & (1 << i2)) > 0);
            }
        }
        Ok(values)
    }

    pub fn read_bytes(&mut self, length: usize) -> DecodeResult<Vec<u8>> {
        if self.cursor.remaining() < length {
            return Err(DecodeError::MissingData);
        }
        let mut bytes = vec![0u8; length];
        self.cursor.read_exact(&mut bytes).unwrap();
        Ok(bytes)
    }

    pub fn read_type<T>(&mut self) -> DecodeResult<T>
    where
        T: Decodable<T>,
    {
        return T::decode(self);
    }

    pub fn decode<T>(buffer: &'a [u8]) -> DecodeResult<T>
    where
        T: Decodable<T>,
    {
        let mut decoder = Self::new(buffer);
        let value: T = decoder.read_type()?;
        return Ok(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        let mut encoder = Encoder::new();
        encoder.write_u8(0xAA);
        encoder.write_u16(0xBBCC);
        encoder.write_bytes(&vec![1, 2, 3]);
        encoder.write_string("Test1");

        assert_eq!(encoder.position(), 12);

        let bytes = encoder.finish();

        assert_eq!(bytes.len(), 12);

        let mut decoder = Decoder::new(&bytes);

        assert_eq!(decoder.position(), 0);
        assert_eq!(decoder.remaining(), 12);

        assert_eq!(decoder.read_u8(), Ok(0xAA));
        assert_eq!(decoder.read_u16(), Ok(0xBBCC));
        assert_eq!(decoder.read_bytes(3), Ok(vec![1, 2, 3]));
        assert_eq!(decoder.read_string(), Ok("Test1".into()));

        assert_eq!(decoder.position(), 12);
        assert_eq!(decoder.remaining(), 0);
    }
}
