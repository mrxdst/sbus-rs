use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct ReadRegistersRequest {
    pub address: u16,
    pub length: u8,
}

impl Encodable for ReadRegistersRequest {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_u8(self.length.checked_sub(1).ok_or(EncodeError::Overflow)?);
        encoder.write_u16(self.address);
        Ok(())
    }
}

impl Decodable<Self> for ReadRegistersRequest {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        Ok(Self {
            length: decoder
                .read_u8()?
                .checked_add(1)
                .ok_or_else(|| DecodeError::InvalidData("Invalid length".into()))?,
            address: decoder.read_u16()?,
        })
    }
}
