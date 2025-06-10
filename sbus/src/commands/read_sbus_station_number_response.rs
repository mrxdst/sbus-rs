use crate::encoding::*;

#[derive(PartialEq, Debug)]
pub struct ReadSBusStationNumberResponse {
    pub station: u8,
}

impl Encodable for ReadSBusStationNumberResponse {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_u8(self.station);
        return Ok(());
    }
}

impl Decodable<Self> for ReadSBusStationNumberResponse {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        return Ok(Self { station: decoder.read_u8()? });
    }
}
