pub fn hex(mut n: u8) -> [u8; 2] {
    let mut hex = [0; 2];
    for i in &[1, 0] {
        hex[*i] = match n % 16 {
            0x0 => 0b01111110,
            0x1 => 0b00110000,
            0x2 => 0b01101101,
            0x3 => 0b01111001,
            0x4 => 0b00110011,
            0x5 => 0b01011011,
            0x6 => 0b01011111,
            0x7 => 0b01110000,
            0x8 => 0b01111111,
            0x9 => 0b01111011,
            0xA => 0b01110111,
            0xb => 0b00011111,
            0xC => 0b01001110,
            0xd => 0b00111101,
            0xE => 0b01001111,
            0xF => 0b01000111,
            _ => 0b10000000,
        };
        n /= 16
    }
    hex
}

/// Write bytes as hex to the display
fn write_hex_bytes<C: max7219::connectors::Connector>(
    display: &mut MAX7219<C>,
    bytes: [u8; 4],
) -> Result<(), DataError> {
    let mut s = [0b10000000; 8];
    let slice = &hex(bytes[0]);
    s[7] = slice[0];
    s[6] = slice[1];
    let slice = &hex(bytes[1]);
    s[5] = slice[0];
    s[4] = slice[1];
    let slice = &hex(bytes[2]);
    s[3] = slice[0];
    s[2] = slice[1];
    let slice = &hex(bytes[3]);
    s[1] = slice[0];
    s[0] = slice[1];

    display.power_off()?;
    display.write_raw(0, &s)?;
    display.power_on()?;
    Ok(())
}
