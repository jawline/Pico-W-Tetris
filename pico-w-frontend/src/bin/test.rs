#![no_std]
#![no_main]

use crate::hal::gpio::bank0::{Gpio2, Gpio3, Gpio4, Gpio5, Gpio6, Gpio7};
use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_alloc::Heap;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{BinaryColor, PixelColor},
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_time::fixed_point::FixedPoint;
use embedded_time::rate::Extensions;
use hal::{
    clocks::{init_clocks_and_plls, Clock},
    i2c::I2C,
    pac,
    watchdog::Watchdog,
    Sio,
};
use panic_probe as _;
use rp2040_hal as hal;
use rp2040_hal::gpio::Pin;
use rp2040_hal::gpio::PullDownInput;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};
use tetris_core::tetris::{KeyState, Tetris};

#[global_allocator]
static HEAP: Heap = Heap::empty();

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

struct Buttons {
    pub up: Pin<Gpio2, PullDownInput>,
    pub left: Pin<Gpio3, PullDownInput>,
    pub down: Pin<Gpio4, PullDownInput>,
    pub right: Pin<Gpio5, PullDownInput>,
    pub a: Pin<Gpio6, PullDownInput>,
    pub b: Pin<Gpio7, PullDownInput>,
}

impl Buttons {
    pub fn a_pressed(&self) -> bool {
        self.a.is_high().unwrap()
    }

    #[allow(dead_code)]
    pub fn b_pressed(&self) -> bool {
        self.b.is_high().unwrap()
    }

    #[allow(dead_code)]
    pub fn up_pressed(&self) -> bool {
        self.up.is_high().unwrap()
    }

    #[allow(dead_code)]
    pub fn down_pressed(&self) -> bool {
        self.down.is_high().unwrap()
    }

    pub fn left_pressed(&self) -> bool {
        self.left.is_high().unwrap()
    }

    pub fn right_pressed(&self) -> bool {
        self.right.is_high().unwrap()
    }
}

#[entry]
fn main() -> ! {
    //Allocator
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

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

    let mut btn_pwr = pins.gpio0.into_push_pull_output();
    btn_pwr.set_high().unwrap();

    let buttons = Buttons {
        up: pins.gpio2.into_pull_down_input(),
        left: pins.gpio3.into_pull_down_input(),
        down: pins.gpio4.into_pull_down_input(),
        right: pins.gpio5.into_pull_down_input(),
        a: pins.gpio6.into_pull_down_input(),
        b: pins.gpio7.into_pull_down_input(),
    };

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let i2c = I2C::i2c1(
        pac.I2C1,
        pins.gpio10.into_mode(), // I2C 1 SDA
        pins.gpio11.into_mode(), // I2C 1 SCL
        400_u32.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
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

    let mut last_led = false;

    let mut tetris = Tetris::new();

    loop {
        let key_state = KeyState {
            left: buttons.left_pressed(),
            right: buttons.right_pressed(),
            rotate: buttons.a_pressed(),
        };
        tetris.set_key_state(&key_state);
        tetris.update();

        screen.clear();

        match tetris {
            Tetris::Running(ref state) => {
                state.draw_game_grid(
                    |x, y, v| {
                        screen.display.set_pixel(x as u32, y as u32, v);
                    },
                    (2, 1),
                    (2, 2),
                );
            }
            Tetris::Finished => tetris = Tetris::new(),
        }

        if last_led {
            led_pin.set_low().unwrap();
        } else {
            led_pin.set_high().unwrap();
        }
        last_led = !last_led;

        screen.flush();
        delay.delay_ms(500);
    }
}
