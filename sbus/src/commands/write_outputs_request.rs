use std::borrow::Cow;

use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct WriteOutputsRequest<'a> {
    pub address: u16,
    pub values: Cow<'a, [bool]>,
}

impl<'a> Encodable for WriteOutputsRequest<'a> {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        let byte_length: u8 = self.values.len().div_ceil(8).try_into()?;
        encoder.write_u8(byte_length.checked_add(2).ok_or(EncodeError::Overflow)?);
        encoder.write_u16(self.address);
        encoder.write_u8(self.values.len().checked_sub(1).ok_or(EncodeError::Overflow)?.try_into()?);
        encoder.write_bools(&self.values);
        Ok(())
    }
}

impl<'a> Decodable<Self> for WriteOutputsRequest<'a> {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        let byte_length = decoder
            .read_u8()?
            .checked_sub(2)
            .ok_or(DecodeError::InvalidData("Invalid byte length"))?;
        let address = decoder.read_u16()?;
        let length = decoder
            .read_u8()?
            .checked_add(1)
            .ok_or(DecodeError::InvalidData("Invalid length"))?;
        let mut values = decoder.read_bools(byte_length as usize)?;
        values.truncate(length as usize);

        Ok(Self {
            address,
            values: values.into(),
        })
    }
}
