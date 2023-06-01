#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};
use embedded_time::fixed_point::FixedPoint;
use embedded_time::rate::Extensions;
use panic_probe as _;
use rp2040_hal as hal;

use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use hal::{
    clocks::{init_clocks_and_plls, Clock},
    i2c::I2C,
    pac,
    watchdog::Watchdog,
    Sio,
};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{BinaryColor, PixelColor},
    prelude::*,
    text::{Baseline, Text},
};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

struct Dim2 {
    pub width: u32,
    pub height: u32,
}

pub struct Screen<'a, DI, SIZE, MODE, C: PixelColor> {
    display: Ssd1306<DI, SIZE, MODE>,
    dim: Dim2,
    text_style: MonoTextStyle<'a, C>,
}

impl<'a, DI: WriteOnlyDataCommand, SIZE: ssd1306::prelude::DisplaySize>
    Screen<'a, DI, SIZE, BufferedGraphicsMode<SIZE>, BinaryColor>
{
    pub fn draw_row(&mut self, row: u32) {
        for i in 0..self.dim.width {
            self.display.set_pixel(i, row, true);
        }
    }

    pub fn clear(&mut self) {
        for x in 0..self.dim.width {
            for y in 0..self.dim.height {
                self.display.set_pixel(x, y, false);
            }
        }
    }

    pub fn text(&mut self, text: &str, point: Point) {
        Text::with_baseline(text, point, self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn flush(&mut self) {
        self.display.flush().unwrap();
    }
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led_pin = pins.gpio25.into_push_pull_output();

    let mut button_one_poll = pins.gpio19.into_pull_down_input();
    let mut button_one_pwr = pins.gpio20.into_push_pull_output();

    button_one_pwr.set_high().unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let i2c = I2C::i2c1(
        pac.I2C1,
        pins.gpio10.into_mode(), // I2C 1 SDA
        pins.gpio11.into_mode(), // I2C 1 SCL
        400_u32.kHz(),
        &mut pac.RESETS,
        125_000_000_u32.Hz(),
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    led_pin.set_high().unwrap();

    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let mut screen = Screen {
        display,
        dim: Dim2 {
            width: 128,
            height: 64,
        },
        text_style,
    };

    let mut current = Point::new(0, 16);
    let mut last_led = false;

    loop {
        screen.clear();
        screen.draw_row(0);
        screen.draw_row(screen.dim.height - 1);

        if button_one_poll.is_high().unwrap() {
            screen.text("Goodbye World", current);
        } else {
            screen.text("Hello World", current);
        }
        screen.flush();

        if last_led {
            led_pin.set_high().unwrap();
        } else {
            led_pin.set_low().unwrap();
        }

        current.x += 1;

        if current.x > 160 {
            current.x = -60;
        }
        last_led = !last_led;
        delay.delay_ms(50);
    }
}
