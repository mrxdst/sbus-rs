use num_enum::{FromPrimitive, IntoPrimitive};

use crate::{encoding::*, utils::crc16};

#[repr(u8)]
#[derive(PartialEq, Debug, Clone, Copy, IntoPrimitive, FromPrimitive)]
pub enum TelegramAttribute {
    Request = 0,
    Response = 1,
    Acknowledge = 2,
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(PartialEq, Debug)]
pub struct Message {
    pub sequence_number: u16,
    pub telegram_attribute: TelegramAttribute,
    pub body: Vec<u8>,
}

impl Encodable for Message {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        let mut pre_encoder = Encoder::new();

        pre_encoder.write_u32((self.body.len() + 11).try_into()?);
        pre_encoder.write_u8(0x01); // Version
        pre_encoder.write_u8(0x00); // Protocol type
        pre_encoder.write_u16(self.sequence_number);
        pre_encoder.write_u8(self.telegram_attribute.into());
        pre_encoder.write_bytes(&self.body);

        let bytes = pre_encoder.finish();

        encoder.write_bytes(&bytes);
        encoder.write_u16(crc16(&bytes));

        Ok(())
    }
}

impl Decodable<Self> for Message {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        let byte_length = decoder.read_u32()?;

        let bytes = decoder.read_bytes(
            byte_length
                .checked_sub(6)
                .ok_or(DecodeError::InvalidData("Invalid byte length"))? as usize,
        )?;

        let checksum = decoder.read_u16()?;
        let mut to_check = Encoder::new();
        to_check.write_u32(byte_length);
        to_check.write_bytes(&bytes);

        if crc16(&to_check.finish()) != checksum {
            return Err(DecodeError::InvalidData("Checksum mismatch"));
        }

        let mut post_decoder = Decoder::new(&bytes);
        post_decoder.read_u8()?; // Version
        post_decoder.read_u8()?; // Protocol type
        let sequence_number = post_decoder.read_u16()?;
        let telegram_attribute = post_decoder.read_u8()?.into();
        let body = post_decoder.read_bytes(post_decoder.remaining())?;

        Ok(Self {
            sequence_number,
            telegram_attribute,
            body,
        })
    }
}
