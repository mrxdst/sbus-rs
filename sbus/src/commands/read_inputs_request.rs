use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct ReadInputsRequest {
    pub address: u16,
    pub length: u8,
}

impl Encodable for ReadInputsRequest {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_u8(self.length.checked_sub(1).ok_or_else(|| EncodeError::Overflow)?);
        encoder.write_u16(self.address);
        return Ok(());
    }
}

impl Decodable<Self> for ReadInputsRequest {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        return Ok(Self {
            length: decoder
                .read_u8()?
                .checked_add(1)
                .ok_or_else(|| DecodeError::InvalidData("Invalid length".into()))?,
            address: decoder.read_u16()?,
        });
    }
}
