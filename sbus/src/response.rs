use std::borrow::Cow;

use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct Response<'a> {
    pub body: Cow<'a, [u8]>,
}

impl<'a> Encodable for Response<'a> {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_bytes(&self.body);
        return Ok(());
    }
}

impl<'a> Decodable<Self> for Response<'a> {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        return Ok(Self {
            body: decoder.read_bytes(decoder.remaining())?.into(),
        });
    }
}
