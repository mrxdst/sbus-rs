use std::borrow::Cow;

use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct ReadFirmwareVersionResponse<'a> {
    pub version: Cow<'a, str>,
}

impl<'a> Encodable for ReadFirmwareVersionResponse<'a> {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_string(&self.version);
        Ok(())
    }
}

impl<'a> Decodable<Self> for ReadFirmwareVersionResponse<'a> {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        Ok(Self {
            version: decoder.read_string()?.into(),
        })
    }
}
