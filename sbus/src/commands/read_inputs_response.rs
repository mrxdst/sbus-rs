use std::borrow::Cow;

use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct ReadInputsResponse<'a> {
    pub values: Cow<'a, [bool]>,
}

impl<'a> Encodable for ReadInputsResponse<'a> {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_bools(&self.values);
        return Ok(());
    }
}

impl<'a> Decodable<Self> for ReadInputsResponse<'a> {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        return Ok(Self {
            values: decoder.read_bools(decoder.remaining() * 8)?.into(),
        });
    }
}
