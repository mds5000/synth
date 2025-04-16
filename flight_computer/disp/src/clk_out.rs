use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{CornerRadii, Line, PrimitiveStyle, Rectangle, RoundedRectangle, StyledDrawable},
};

use crate::Display;
use crate::{
    display::{FONT_16, TAN},
    output::OutSignal,
};
use crate::{
    output::{NoOutput, OutputChannel},
    parameters::Parameter,
};

#[derive(Clone)]
pub struct ClockOut {
    multiplier: Parameter<i32>,
    duty_cycle: Parameter<f32>,
}

#[derive(Clone, Copy)]
pub struct ClockData {
    last_edge_cycle: u32,
}

impl Default for ClockOut {
    fn default() -> Self {
        ClockOut {
            multiplier: Parameter::new_saturating(-64, 64, 1, 1),
            duty_cycle: Parameter::new_saturating(0.0, 1.0, 0.05, 0.5),
        }
    }
}

impl OutSignal for ClockOut {
    fn next(&self) -> OutputChannel {
        NoOutput::new().into()
    }

    fn prev(&self) -> OutputChannel {
        NoOutput::new().into()
    }

    fn num_parameters(&self) -> usize {
        2
    }

    fn parameter(
        &mut self,
        param: usize,
    ) -> Option<(&'static str, &mut dyn crate::parameters::ConfigParameter)> {
        match param {
            0 => Some(("Division", &mut self.multiplier)),
            1 => Some(("Duty Cycle", &mut self.duty_cycle)),
            _ => None,
        }
    }

    fn draw_output(&self, disp: &mut Display, window: Rectangle) {
        let line_g = PrimitiveStyle::with_stroke(Bgr565::CSS_GHOST_WHITE, 1);
        let center_y = window.center().y;
        let line_start = Point::new(window.top_left.x + 2, center_y);
        let line_end = Point::new(window.top_left.x + 28, center_y);

        Line::new(line_start, line_end).draw_styled(&line_g, disp);
    }

    fn draw_configure(&self, disp: &mut Display, window: Rectangle) {
        let green = PrimitiveStyle::with_stroke(Bgr565::GREEN, 1);
        window.offset(-1).draw_styled(&green, disp);

        let anchor = window.top_left + Point::new(5, 5);
        FONT_16.render_aligned(
            format_args!("Clock Out"),
            anchor,
            u8g2_fonts::types::VerticalPosition::Top,
            u8g2_fonts::types::HorizontalAlignment::Left,
            u8g2_fonts::types::FontColor::Transparent(TAN),
            disp,
        );
    }
}

enum Multiplier {
    x64,
    x32,
    x16,
    x8,
    x4,
    x3,
    x2,
    x1,
    div2,
    div3,
    div4,
    div5,
    div6,
    div7,
    div8,
    div16,
}

impl Multiplier {
    pub fn as_f32(&self) -> f32 {
        match self {
            Multiplier::x64 => 64.0,
            Multiplier::x32 => 32.0,
            Multiplier::x16 => 16.0,
            Multiplier::x8 => 8.0,
            Multiplier::x4 => 4.0,
            Multiplier::x3 => 3.0,
            Multiplier::x2 => 2.0,
            Multiplier::x1 => 1.0,
            Multiplier::div2 => 0.5,
            Multiplier::div3 => 1.0 / 3.0,
            Multiplier::div4 => 0.25,
            Multiplier::div5 => 0.2,
            Multiplier::div6 => 1.0 / 6.0,
            Multiplier::div7 => 1.0 / 7.0,
            Multiplier::div8 => 0.125,
            Multiplier::div16 => 0.0625,
        }
    }

    pub fn as_ratio(&self) -> (u16, u16) {
        match self {
            Multiplier::x64 => (64, 0),
            Multiplier::x32 => (32, 0),
            Multiplier::x16 => (16, 0),
            Multiplier::x8 => (8, 0),
            Multiplier::x4 => (4, 0),
            Multiplier::x3 => (3, 0),
            Multiplier::x2 => (2, 0),
            Multiplier::x1 => (1, 0),
            Multiplier::div2 => (0, 2),
            Multiplier::div3 => (0, 3),
            Multiplier::div4 => (0, 4),
            Multiplier::div5 => (0, 5),
            Multiplier::div6 => (0, 6),
            Multiplier::div7 => (0, 7),
            Multiplier::div8 => (0, 8),
            Multiplier::div16 => (0, 16),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Multiplier::x64 => Multiplier::x64,
            Multiplier::x32 => Multiplier::x64,
            Multiplier::x16 => Multiplier::x32,
            Multiplier::x8 => Multiplier::x16,
            Multiplier::x4 => Multiplier::x8,
            Multiplier::x3 => Multiplier::x4,
            Multiplier::x2 => Multiplier::x3,
            Multiplier::x1 => Multiplier::x2,
            Multiplier::div2 => Multiplier::x1,
            Multiplier::div3 => Multiplier::div2,
            Multiplier::div4 => Multiplier::div3,
            Multiplier::div5 => Multiplier::div4,
            Multiplier::div6 => Multiplier::div5,
            Multiplier::div7 => Multiplier::div6,
            Multiplier::div8 => Multiplier::div7,
            Multiplier::div16 => Multiplier::div8,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Multiplier::x64 => Multiplier::x32,
            Multiplier::x32 => Multiplier::x16,
            Multiplier::x16 => Multiplier::x8,
            Multiplier::x8 => Multiplier::x4,
            Multiplier::x4 => Multiplier::x3,
            Multiplier::x3 => Multiplier::x2,
            Multiplier::x2 => Multiplier::x1,
            Multiplier::x1 => Multiplier::div2,
            Multiplier::div2 => Multiplier::div3,
            Multiplier::div3 => Multiplier::div4,
            Multiplier::div4 => Multiplier::div5,
            Multiplier::div5 => Multiplier::div6,
            Multiplier::div6 => Multiplier::div7,
            Multiplier::div7 => Multiplier::div8,
            Multiplier::div8 => Multiplier::div16,
            Multiplier::div16 => Multiplier::div16,
        }
    }
}
