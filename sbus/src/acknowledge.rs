use num_enum::{FromPrimitive, IntoPrimitive};

use crate::encoding::*;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive, FromPrimitive)]
pub enum Acknowledge {
    Ack = 0,
    Nak = 1,
    NakPassword = 2,
    NakPGUReducedProtocol = 3,
    NakPGUAlreadyUsed = 4,
    #[num_enum(catch_all)]
    Unknown(u16),
}

impl Encodable for Acknowledge {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_u16((*self).into());
        return Ok(());
    }
}

impl Decodable<Self> for Acknowledge {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        Ok(decoder.read_u16()?.into())
    }
}
