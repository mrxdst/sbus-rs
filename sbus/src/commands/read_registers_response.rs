use std::borrow::Cow;

use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct ReadRegistersResponse<'a> {
    pub values: Cow<'a, [i32]>,
}

impl<'a> Encodable for ReadRegistersResponse<'a> {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.reserve(self.values.len());
        for value in self.values.iter() {
            encoder.write_i32(*value);
        }
        return Ok(());
    }
}

impl<'a> Decodable<Self> for ReadRegistersResponse<'a> {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        let mut values = Vec::with_capacity(decoder.remaining() / 4);
        while decoder.remaining() > 0 {
            values.push(decoder.read_i32()?);
        }
        return Ok(Self { values: values.into() });
    }
}
