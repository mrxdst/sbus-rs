use std::borrow::Cow;

use crate::{command_id::CommandId, encoding::*};

#[derive(PartialEq, Debug)]
pub struct Request<'a> {
    pub station: u8,
    pub command_id: CommandId,
    pub body: Cow<'a, [u8]>,
}

impl<'a> Encodable for Request<'a> {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_u8(self.station);
        encoder.write_u8(self.command_id.into());
        encoder.write_bytes(&self.body);
        return Ok(());
    }
}

impl<'a> Decodable<Self> for Request<'a> {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        return Ok(Self {
            station: decoder.read_u8()?,
            command_id: decoder.read_u8()?.into(),
            body: decoder.read_bytes(decoder.remaining())?.into(),
        });
    }
}
