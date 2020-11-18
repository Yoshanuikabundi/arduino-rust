#![no_std]
#![no_main]

/// Read the temperature from a dht11 and print it to a max7219 display
extern crate panic_halt;
use arduino_uno::hal::port::mode::*;
use arduino_uno::hal::port::Pin;
use arduino_uno::prelude::*;
use dht11::Dht11;
use max7219::*;

#[allow(dead_code)]
fn def<T: Default>() -> T {
    T::default()
}

trait Error {}
impl Error for max7219::DataError {}
struct GenericError;
impl<T: Error> From<T> for GenericError {
    fn from(_: T) -> Self {
        Self
    }
}

/// Takes a temperature in tenths of a degree C and displays it
fn display_temperature<C>(temp: i16, display: &mut MAX7219<C>) -> Result<(), max7219::DataError>
where
    C: max7219::connectors::Connector,
{
    let mut bytes = [0_u8; 8];

    bytes[0] = 0b01001110; // C as last digit
    bytes[1] = 0b00000000; // Space between number and unit
    bytes[2] = dec_digit(temp);
    let mut v = temp / 10;
    bytes[3] = dec_digit(v) | 0b10000000; // Units column, so add a dot

    for i in 4..8 {
        v /= 10;
        bytes[i] = if v != 0 {
            dec_digit(v)
        } else if temp < 0 {
            0b00000001
        } else {
            0b00000000 // sign (- or blank)
        };
    }

    display.power_off()?;
    display.write_raw(0, &bytes)?;
    display.power_on()?;
    Ok(())
}

// Decimal representation of least significant digit
pub fn dec_digit(n: i16) -> u8 {
    match n % 10 {
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
        _ => 0b10000000,
    }
}

fn blink_high_ms(pin: &mut Pin<Output>, ms: u16) {
    let _ = pin.set_high().void_unwrap();
    arduino_uno::delay_ms(ms);
    let _ = pin.set_low().void_unwrap();
}

fn run(pins: arduino_uno::Pins) -> Result<(), GenericError> {
    let mut display = {
        let clk = pins.d13.into_output(&pins.ddr);
        let cs = pins.d12.into_output(&pins.ddr);
        let din = pins.d11.into_output(&pins.ddr);

        MAX7219::from_pins(1, din, cs, clk)?
    };

    let mut temp_hmdty_sensor = Dht11::new(pins.d8.into_tri_state(&pins.ddr));
    let mut led = pins.d2.into_output(&pins.ddr).downgrade();

    while display.write_str(0, b"   hello", 0b11100000).is_err() {}
    display.set_intensity(0, 0xFF);
    display.power_on()?;

    // Make a bunch of measurements
    let time_between_updates = 20_000;
    let mut temps = [None; 15];
    let calibration_offset = -5;
    let time_per_loop = time_between_updates / temps.len() as u16;
    loop {
        for i in 0..temps.len() {
            let sensor_output =
                temp_hmdty_sensor.perform_measurement(&mut arduino_uno::Delay::new());
            if let Ok(m) = sensor_output {
                temps[i] = Some(m.temperature);
            };

            if i == 0 {
                blink_high_ms(&mut led, time_per_loop);
            } else {
                arduino_uno::delay_ms(time_per_loop)
            }
        }

        // Average over measurements
        let t = temps.iter().filter_map(|&t| t).fold(None, |a, t| {
            if let Some((a, count)) = a {
                Some((a + t, count + 1))
            } else {
                Some((t, 1))
            }
        });

        // Display the average if there was at least one measurement
        if let Some((t, count)) = t {
            let t = (t / count as i16) + calibration_offset;
            let _ = display_temperature(t, &mut display);
        }
    }

    #[allow(unreachable_code)]
    Ok(())
}

#[arduino_uno::entry]
fn main() -> ! {
    let dp = arduino_uno::Peripherals::take().unwrap();
    let pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

    if run(pins).is_err() {
        panic!()
    };

    loop {}
}
