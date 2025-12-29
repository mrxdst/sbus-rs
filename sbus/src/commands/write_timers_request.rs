use std::borrow::Cow;

use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct WriteTimersRequest<'a> {
    pub address: u16,
    pub values: Cow<'a, [i32]>,
}

impl<'a> Encodable for WriteTimersRequest<'a> {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_u8((self.values.len() * 4 + 1).try_into()?);
        encoder.write_u16(self.address);
        encoder.reserve(self.values.len());
        for value in self.values.iter() {
            encoder.write_i32(*value);
        }
        Ok(())
    }
}

impl<'a> Decodable<Self> for WriteTimersRequest<'a> {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        let length = decoder
            .read_u8()?
            .checked_sub(1)
            .ok_or(DecodeError::InvalidData("Invalid length"))?;
        if length % 4 != 0 {
            return Err(DecodeError::InvalidData("Invalid length"));
        }
        let length = length / 4;
        let address = decoder.read_u16()?;
        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length {
            values.push(decoder.read_i32()?);
        }
        Ok(Self {
            address,
            values: values.into(),
        })
    }
}
