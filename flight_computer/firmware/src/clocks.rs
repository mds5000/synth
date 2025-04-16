// Output Modes
//  - Clk  
        - div/mult

struct ClockOut {
    multiplier: f32,
    trigger: bool,
    skip_probability: f32,
}

struct EuclidianOut {
    steps: u32,
    pulses: u32,
    offset: u32,
}

struct LowFreqOscillator {
    frequency: f32,
    amplitude: f32,
    offset: f32,
    phase: f32,
}

struct SmoothRandom {

}

struct SteppedRandom {

}

struct MidiPitch {
    channel: u32,
    order: u32,
    slew: f32,
}

struct MidiControl {
    channel: u32,
    slew: f32,
}