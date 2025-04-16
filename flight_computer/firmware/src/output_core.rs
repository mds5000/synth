use core::cell::RefCell;
use core::sync::atomic::AtomicU32;

use cortex_m_rt::exception;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use rp235x_hal::pio::{PIOExt, PinDir};
use rp235x_hal as hal;
use rp235x_hal::pac;

use critical_section;

use crate::LED;

static mut CYCLE: AtomicU32 = AtomicU32::new(0);

pub fn core1_loop() {
    let core = unsafe { cortex_m::Peripherals::steal() };
    let mut pac = unsafe { pac::Peripherals::steal() };
    let sys_clk = 150_000_000u32;

    let mut led = critical_section::with(|cs| LED.borrow(cs).take()).unwrap();
    led.set_high();

    let (mut pio, sm0, sm1, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let program = pio_proc::pio_file!("./pio/dac.pio");

    let installed_program = pio.install(&program.program).unwrap();
    let (mut sm0, _, mut sm0_tx) = hal::pio::PIOBuilder::from_installed_program(unsafe {installed_program.share()})
        .clock_divisor_fixed_point(4, 0)
        .set_pins(16,1)  // DAC_SYNC
        .out_pins(18, 1) // DAC_DATA
        .side_set_pin_base(17)       // DAC_CLK
        .in_shift_direction(hal::pio::ShiftDirection::Right)
        .out_shift_direction(hal::pio::ShiftDirection::Right)
        .pull_threshold(24)
        .autopull(false)
        .build(sm0);
    sm0.clear_fifos();
    sm0.set_pindirs([
        (16, PinDir::Output),
        (17, PinDir::Output),
        (18, PinDir::Output),
    ]);

    let (mut sm1, _, mut sm1_tx) = hal::pio::PIOBuilder::from_installed_program(unsafe {installed_program.share()})
        .clock_divisor_fixed_point(4, 0)
        .set_pins(19,1)  // DAC_SYNC
        .out_pins(21, 1) // DAC_DATA
        .side_set_pin_base(20)       // DAC_CLK
        .in_shift_direction(hal::pio::ShiftDirection::Right)
        .out_shift_direction(hal::pio::ShiftDirection::Right)
        .pull_threshold(24)
        .autopull(false)
        .build(sm1);
    sm1.clear_fifos();
    sm1.set_pindirs([
        (19, PinDir::Output),
        (20, PinDir::Output),
        (21, PinDir::Output),
    ]);

    let s = sm0.with(sm1).sync().start();

    /*
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led = pins.gpio25.into_push_pull_output();
    */
    let mut systimer = core.SYST;
    systimer.set_reload(3125-1); // 48KHz at 150MHz SYSCLK
    systimer.clear_current();
    systimer.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    systimer.enable_interrupt();
    systimer.enable_counter();

    loop {
        led.set_high();
        let cycle = unsafe {CYCLE.load(core::sync::atomic::Ordering::Relaxed)};
        sm0_tx.write(cycle);
        sm0_tx.write(cycle+1);
        sm0_tx.write(cycle+2);
        sm0_tx.write(cycle+3);
        sm1_tx.write(0);
        sm1_tx.write(1);
        sm1_tx.write(2);
        sm1_tx.write(3);
        pio.force_irq(1);
        led.set_low();
        
        hal::arch::wfi();
    }
}

#[exception]
fn SysTick() {
    unsafe {
    let mut cycle = CYCLE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }
}



