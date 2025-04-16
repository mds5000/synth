#![no_std]
#![no_main]

use core::borrow::BorrowMut;

use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use rp235x_hal::{entry, fugit::RateExtU32, gpio::{Pin, FunctionPio0, FunctionSio, SioOutput, PullDown}, spi::SpiDevice};
use panic_halt as _;
use rp235x_hal as hal;
use embedded_hal::spi::MODE_0;

use hal::{
    pac,
    gpio,
    gpio::{PullNone, FunctionSpi},
    multicore::{Multicore, Stack},
   clocks::{init_clocks_and_plls, Clock},
   watchdog::Watchdog,
   Sio,
};
use ssd1351::{mode::GraphicsMode, prelude::SPIInterface};
use embedded_graphics::{geometry::{Point, Size}, primitives::StyledDrawable};
use embedded_graphics::{primitives::{PrimitiveStyleBuilder, Rectangle}};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};

use core::cell::RefCell;
use critical_section::Mutex;

mod output_core;

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000u32;
static mut CORE1_STACK: Stack<4096> = Stack::new();
pub static LED: Mutex<RefCell<Option<Pin<hal::gpio::bank0::Gpio25, FunctionSio<SioOutput>, PullDown>>>> = Mutex::new(RefCell::new(None));

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = cortex_m::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    let mut sio = Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.gpio25.into_push_pull_output();
    critical_section::with(|cs| LED.borrow(cs).replace(Some(led_pin)));

    // Setup PIO GPIO Pins
    let _dac1_sync: Pin<_, FunctionPio0, _> = pins.gpio16.into_function();
    let _dac1_clk: Pin<_, FunctionPio0, _> = pins.gpio17.into_function();
    let _dac1_data: Pin<_, FunctionPio0, _> = pins.gpio18.into_function();
    let _dac2_sync: Pin<_, FunctionPio0, _> = pins.gpio19.into_function();
    let _dac2_clk: Pin<_, FunctionPio0, _> = pins.gpio20.into_function();
    let _dac2_data: Pin<_, FunctionPio0, _> = pins.gpio21.into_function();
    let mut dac_rst = pins.gpio22.into_push_pull_output();
    dac_rst.set_high();

    let disp_clk: gpio::Pin<_, FunctionSpi, PullNone> = pins.gpio2.reconfigure();
    let disp_mosi: gpio::Pin<_, FunctionSpi, PullNone> = pins.gpio3.reconfigure();
    let disp_cs = pins.gpio5.into_push_pull_output();
    let mut disp_rst = pins.gpio6.into_push_pull_output();
    let disp_dc = pins.gpio7.into_push_pull_output();
    let mut disp_spi = hal::spi::Spi::<_,_,_,8>::new(pac.SPI0, (disp_mosi, disp_clk))
        .init(&mut pac.RESETS, clocks.system_clock.freq(), 50_000_000u32.Hz(), MODE_0);
    let disp_dev = ExclusiveDevice::new_no_delay(disp_spi, disp_cs).unwrap();
    let disp_if = SPIInterface::new(disp_dev, disp_dc);
    let mut disp: GraphicsMode<_> = ssd1351::builder::Builder::new()
        .with_rotation(ssd1351::properties::DisplayRotation::Rotate0)
        .with_size(ssd1351::properties::DisplaySize::Display128x128)
        .connect_interface(disp_if)
        .into();

    disp.reset(&mut disp_rst, &mut timer).unwrap();
    disp.init().unwrap();

    let style = PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build();


    let system_clk = clocks.system_clock.freq().to_Hz();
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    core1.spawn(unsafe {&mut CORE1_STACK.mem}, output_core::core1_loop);


    loop {
        Rectangle::new(Point::new(0,0), Size::new(128, 128)).draw_styled(&style, &mut disp);
    }
}