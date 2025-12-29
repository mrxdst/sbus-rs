use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct ReadInputsRequest {
    pub address: u16,
    pub length: u8,
}

impl Encodable for ReadInputsRequest {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_u8(self.length.checked_sub(1).ok_or(EncodeError::Overflow)?);
        encoder.write_u16(self.address);
        Ok(())
    }
}

impl Decodable<Self> for ReadInputsRequest {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        Ok(Self {
            length: decoder
                .read_u8()?
                .checked_add(1)
                .ok_or(DecodeError::InvalidData("Invalid length"))?,
            address: decoder.read_u16()?,
        })
    }
}
