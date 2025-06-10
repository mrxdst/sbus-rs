use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct ReadDisplayRegisterResponse {
    pub register: u32,
}

impl Encodable for ReadDisplayRegisterResponse {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_u32(self.register);
        return Ok(());
    }
}

impl Decodable<Self> for ReadDisplayRegisterResponse {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        return Ok(Self {
            register: decoder.read_u32()?,
        });
    }
}
