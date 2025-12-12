pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u32 = 0;

    for byte in data {
        crc ^= (*byte as u32) << 8;
        for _ in 0..8 {
            crc <<= 1;
            if crc & 0x10000 != 0 {
                crc = (crc ^ 0x1021) & 0xFFFF;
            }
        }
    }

    crc as u16
}

/// Converts a 32-bit S-Bus float into [f64].
/// The result will always be finite.
pub fn sbus_float_to_ieee(value: i32) -> f64 {
    // mmmmmmmmmmmmmmmmmmmmmmmmseeeeeee    s=7 e=0-6 m=8-31
    // exponent x-64
    // mantissa x / 2**24
    let value = u32::from_ne_bytes(value.to_ne_bytes());

    let s: f64 = if value & 0x80 != 0 { -1.0 } else { 1.0 };
    let e: f64 = ((value & 0x7F) as i8 - 64) as f64;
    let m: f64 = (value >> 8) as f64 / 16777216.0;

    s * f64::powf(2.0, e) * m
}

/// Converts [f64] into a 32-bit S-Bus float.
/// `NaN` will be mapped to `0`. `±Infinity` will be mapped to the most positive or negative value.
pub fn ieee_to_sbus_float(value: f64) -> i32 {
    if value.is_nan() {
        return 0; // Best we can do
    }

    if value.is_infinite() {
        let s: u32 = if f64::signum(value) == -1.0 { 1 << 7 } else { 0 };
        let e: u32 = 0x7F;
        let m: u32 = 0xFFFFFF << 8;
        return i32::from_ne_bytes((s | e | m).to_ne_bytes());
    }

    let s: f64 = f64::signum(value);
    let e: f64 = f64::ceil(f64::log2(value));
    let m: f64 = value / f64::powf(2.0, e);

    let s: u32 = if s == -1.0 { 1 << 7 } else { 0 };
    let e: u32 = f64::clamp(e + 64.0, 0.0, 127.0) as u32;
    let m: u32 = ((m * 16777216.0) as u32) << 8;

    i32::from_ne_bytes((s | e | m).to_ne_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbc_float() {
        let ieee: f64 = 1234.5;
        let sbc: i32 = -1706033077;
        assert_eq!(ieee_to_sbus_float(ieee), sbc);
        assert_eq!(sbus_float_to_ieee(sbc), ieee);

        assert_eq!(sbus_float_to_ieee(ieee_to_sbus_float(f64::INFINITY)), 9.223371487098962e18);
        assert_eq!(sbus_float_to_ieee(ieee_to_sbus_float(f64::NEG_INFINITY)), -9.223371487098962e18);
    }
}
