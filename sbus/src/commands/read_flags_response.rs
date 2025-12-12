use std::borrow::Cow;

use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct ReadFlagsResponse<'a> {
    pub values: Cow<'a, [bool]>,
}

impl<'a> Encodable for ReadFlagsResponse<'a> {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_bools(&self.values);
        Ok(())
    }
}

impl<'a> Decodable<Self> for ReadFlagsResponse<'a> {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        Ok(Self {
            values: decoder.read_bools(decoder.remaining() * 8)?.into(),
        })
    }
}
