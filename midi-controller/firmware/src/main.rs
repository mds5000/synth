#![no_main]
#![no_std]

use embedded_midi::MidiMessage;
use panic_halt as _;

use hal::prelude::*;
use hal::stm32;
use hal::rcc::Config;
use hal::serial::FullConfig;
use stm32g4xx_hal::dac::DacExt;
use stm32g4xx_hal::dac::DacOut;
use stm32g4xx_hal::pwr::PwrExt;
use stm32g4xx_hal::time::{Bps, RateExtU32};
use stm32g4xx_hal as hal;

use embedded_midi::{MidiIn, Note};

use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln, ChannelMode};

fn note_to_voltage(note: Note) -> u16 {
    let full_scale: f32 = 3.300_f32 * 3.0 * 1_000_000.0; //uv full-scale output;
    let codes = 4095.0;
    let uv_per_volt = 1_000_000_f32;
    let notes_per_octave = 12_f32;
    let codes_per_note: f32 = uv_per_volt / (full_scale / codes) / notes_per_octave;

    let note_number: u8 = note.into();
    let code = unsafe { ((note_number as f32) * codes_per_note).to_int_unchecked::<u16>() };
    let bias = 5;

    code + bias
}

#[entry]
fn main() -> ! {
    rtt_init_print!(ChannelMode::BlockIfFull);
    rprintln!("Started");

    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let cp = cortex_m::Peripherals::take().expect("cannot take peripherals");
    let pwr = dp.PWR.constrain().freeze();
    let mut rcc = dp.RCC.constrain().freeze(Config::pll(), pwr);
    let mut delay = cp.SYST.delay(&rcc.clocks);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let mut gate_1 = gpioa.pa7.into_push_pull_output();
    let mut _gate_2 = gpioa.pa6.into_push_pull_output();

    let mod_1_pin = gpioa.pa0.into_alternate();
    let mod_2_pin = gpioa.pa1.into_alternate();
    let (mut mod_1, mut mod_2) = dp.TIM2.pwm((mod_1_pin, mod_2_pin), 100.kHz(), &mut rcc);

    let (dac_ch1, dac_ch2) = dp.DAC1.constrain((gpioa.pa4, gpioa.pa5), &mut rcc);
    let mut cv_1 = dac_ch1.calibrate_buffer(&mut delay).enable();
    let mut cv_2 = dac_ch2.calibrate_buffer(&mut delay).enable();

    let tx = gpioa.pa2.into_alternate();
    let rx = gpioa.pa3.into_alternate();
    let midi_rx = dp.USART2.usart(tx, rx, FullConfig::default().baudrate(Bps(31250)), &mut rcc).unwrap();
    let mut midi = MidiIn::new(midi_rx);

    let gpiob = dp.GPIOB.split(&mut rcc);
    let mut led_1 = gpiob.pb8.into_push_pull_output();
    let mut led_2 = gpiob.pb7.into_push_pull_output();

    led_2.set_high().unwrap();
    cv_1.set_value(2048);
    cv_2.set_value(1024);
    loop {
        if let Ok(msg) = midi.read() {
            match msg {
                MidiMessage::TimingClock => {
                    led_2.toggle().unwrap();
                }
                MidiMessage::NoteOn(_, note, _) => {
                    led_1.set_high().unwrap();
                    gate_1.set_high().unwrap();
                    let value = note_to_voltage(note);
                    cv_1.set_value(value);
                    //rprintln!("NOTE ON: {:?} -> {}", note, value)
                },
                MidiMessage::NoteOff(_, _note, _) => {
                    led_1.set_low().unwrap();
                    gate_1.set_low().unwrap();
                    //rprintln!("NOTE OFF: {:?}", note)
                },
                MidiMessage::ControlChange(_, ch, value) => {
                    let v: u8 = value.into();
                    match ch.into() {
                        0 => mod_1.set_duty(mod_1.get_max_duty() / 128 * (v as u32)),
                        1 => mod_2.set_duty(mod_2.get_max_duty() / 128 * (v as u32)),
                        _ => {}
                    }
                    //rprintln!("PARAM CHG: {:?}: {:?}", ch, value),
                },

                unhandled_msg => rprintln!("OTHER: {:?}", unhandled_msg),
            }
        }
    }
}