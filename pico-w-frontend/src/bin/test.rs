#![no_std]
#![no_main]

use crate::hal::gpio::bank0::*;
use crate::hal::gpio::PushPullOutput;
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
    pub fn draw_row(&mut self, row: u32, (start_x, end_x): (u32, u32)) {
        for i in start_x..end_x {
            self.display.set_pixel(i, row, true);
        }
    }

    pub fn draw_col(&mut self, col: u32, (start_y, end_y): (u32, u32)) {
        for i in start_y..end_y {
            self.display.set_pixel(col, i, true);
        }
    }

    pub fn draw_rect(&mut self, (min_x, min_y): (u32, u32), (max_x, max_y): (u32, u32)) {
        self.draw_row(min_y, (min_x, max_x));
        self.draw_row(max_y, (min_x, max_x));
        self.draw_col(min_x, (min_y, max_y));
        self.draw_col(max_x, (min_y, max_y));
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
    // Left = Gpio22
    // Right = 19 and 18 (Hardware bug, fix)
    // Down = 16
    pub up: Pin<Gpio18, PullDownInput>,
    pub left: Pin<Gpio22, PullDownInput>,
    pub down: Pin<Gpio19, PullDownInput>,
    pub right: Pin<Gpio17, PullDownInput>,
    pub a: Pin<Gpio16, PullDownInput>,
    pub b: Pin<Gpio21, PullDownInput>,
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

fn print_buttons<'a, DI: WriteOnlyDataCommand, SIZE: ssd1306::prelude::DisplaySize>(
    screen: &mut Screen<'a, DI, SIZE, BufferedGraphicsMode<SIZE>, BinaryColor>,
    buttons: &Buttons,
    led_pin: &mut Pin<Gpio25, PushPullOutput>,
) {
    const CHR_SZ_X: i32 = 4;
    let mut btn = false;
    if buttons.left_pressed() {
        screen.text("L", Point::new(screen.dim.width as i32 - CHR_SZ_X, 0));
        btn = true;
    }

    if buttons.right_pressed() {
        screen.text("R", Point::new(screen.dim.width as i32 - (CHR_SZ_X * 2), 0));
        btn = true;
    }

    if buttons.up_pressed() {
        screen.text("U", Point::new(screen.dim.width as i32 - (CHR_SZ_X * 3), 0));
        btn = true;
    }

    if buttons.down_pressed() {
        screen.text("D", Point::new(screen.dim.width as i32 - (CHR_SZ_X * 4), 0));
        btn = true;
    }

    if buttons.a_pressed() {
        screen.text("A", Point::new(screen.dim.width as i32 - (CHR_SZ_X * 5), 0));
        btn = true;
    }

    if buttons.b_pressed() {
        screen.text("B", Point::new(screen.dim.width as i32 - (CHR_SZ_X * 6), 0));
        btn = true;
    }

    if btn {
        led_pin.set_high().unwrap();
    } else {
        led_pin.set_low().unwrap();
    }
}

fn print_tetris<'a, DI: WriteOnlyDataCommand, SIZE: ssd1306::prelude::DisplaySize>(
    screen: &mut Screen<'a, DI, SIZE, BufferedGraphicsMode<SIZE>, BinaryColor>,
    tetris: &Tetris,
) {
    screen.draw_rect((1, 9), (43, 50));
    match tetris {
        Tetris::Running(ref state) => {
            state.draw_game_grid(
                |x, y, v| {
                    screen.display.set_pixel(x as u32, y as u32, v);
                },
                (2, 10),
                (4, 2),
            );
        }
        Tetris::Finished => {}
    }
}

fn update(tetris: &mut Tetris, buttons: &Buttons) {
    let key_state = KeyState {
        left: buttons.left_pressed(),
        right: buttons.right_pressed(),
        rotate: buttons.a_pressed(),
    };

    tetris.set_key_state(&key_state);
    tetris.update();

    match tetris {
        Tetris::Running(ref _state) => {}
        Tetris::Finished => {
            if buttons.b_pressed() {
                *tetris = Tetris::new();
            }
        }
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
        up: pins.gpio18.into_pull_down_input(),
        left: pins.gpio22.into_pull_down_input(),
        down: pins.gpio19.into_pull_down_input(),
        right: pins.gpio17.into_pull_down_input(),
        a: pins.gpio16.into_pull_down_input(),
        b: pins.gpio21.into_pull_down_input(),
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

    let mut tetris = Tetris::new();

    loop {
        update(&mut tetris, &buttons);
        screen.clear();

        print_tetris(&mut screen, &mut tetris);
        print_buttons(&mut screen, &buttons, &mut led_pin);

        screen.flush();
        delay.delay_ms(100);
    }
}
