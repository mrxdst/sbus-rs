use crate::encoding::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct RealTimeClock {
    pub week: u8,
    pub week_day: u8,
    pub year: u8,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Encodable for RealTimeClock {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        let mut write = |v: u8| -> EncodeResult {
            encoder.write_u8(u8::from_str_radix(&v.to_string(), 16).map_err(|_| EncodeError::Overflow)?);
            Ok(())
        };
        write(self.week)?;
        write(self.week_day)?;
        write(self.year)?;
        write(self.month)?;
        write(self.day)?;
        write(self.hour)?;
        write(self.minute)?;
        write(self.second)?;
        return Ok(());
    }
}

impl Decodable<Self> for RealTimeClock {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        let mut read = || -> DecodeResult<u8> {
            let v: u8 = format!("{:X}", decoder.read_u8()?)
                .parse()
                .map_err(|_| DecodeError::InvalidData("Invalid time data".into()))?;
            Ok(v)
        };
        return Ok(Self {
            week: read()?,
            week_day: read()?,
            year: read()?,
            month: read()?,
            day: read()?,
            hour: read()?,
            minute: read()?,
            second: read()?,
        });
    }
}
