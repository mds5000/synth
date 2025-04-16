use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::Rectangle;
use enum_dispatch::enum_dispatch;

use crate::display::{FONT_16, TAN};
use crate::parameters::{ConfigParameter, Parameter};
use crate::Display;

use crate::clk_out::{ClockData, ClockOut};

#[enum_dispatch]
pub trait OutSignal {
    fn num_parameters(&self) -> usize;
    fn parameter(&mut self, param: usize) -> Option<(&'static str, &mut dyn ConfigParameter)>;
    fn draw_output(&self, disp: &mut Display, window: Rectangle);
    fn draw_configure(&self, disp: &mut Display, window: Rectangle);
    fn next(&self) -> OutputChannel;
    fn prev(&self) -> OutputChannel;
    //fn store(&self) -> &[u8];
    //fn load(&mut self, state: &[u8]) {}
    //fn generate(&self, input: InputState, private: &mut PrivateData) -> i16 {}
}

#[enum_dispatch(OutSignal)]
pub enum OutputChannel {
    NoOutput,
    ClockOut,
}

pub union PrivateData {
    clkout: ClockData,
}

#[derive(Clone)]
pub struct NoOutput;

impl NoOutput {
    pub fn new() -> Self {
        NoOutput {}
    }
}

impl OutSignal for NoOutput {
    fn num_parameters(&self) -> usize {
        0
    }

    fn parameter(&mut self, param: usize) -> Option<(&'static str, &mut dyn ConfigParameter)> {
        None
    }

    fn next(&self) -> OutputChannel {
        ClockOut::default().into()
    }

    fn prev(&self) -> OutputChannel {
        ClockOut::default().into()
    }

    fn draw_output(&self, disp: &mut Display, window: Rectangle) {}
    fn draw_configure(&self, disp: &mut Display, window: Rectangle) {
        let anchor = window.top_left + Point::new(5, 5);

        FONT_16.render_aligned(
            format_args!("Disabled"),
            anchor,
            u8g2_fonts::types::VerticalPosition::Top,
            u8g2_fonts::types::HorizontalAlignment::Left,
            u8g2_fonts::types::FontColor::Transparent(TAN),
            disp,
        );
    }
}
