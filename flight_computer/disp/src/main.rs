use std::sync::OnceLock;
use std::{thread, time::Duration};

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::AnchorPoint;
use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::{
    CornerRadii, PrimitiveStyle, Rectangle, RoundedRectangle, StyledDrawable,
};
use embedded_graphics_simulator::{
    sdl2::Keycode, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

mod clk_out;
mod display;
mod output;
mod parameters;

use crate::display::{Display, FONT_08, FONT_10, FONT_16};
use crate::display::{BG, BLUE, BRIGHT, DARK, TAN};

use core::iter::zip;
use output::{NoOutput, OutSignal, OutputChannel};

static mut OUTPUTS: OnceLock<[OutputChannel; 8]> = OnceLock::new();

fn add_wrap(a: u8, b: i8, max: u8) -> u8 {
    if a == (max - 1) && b > 0 {
        0
    } else if a == 0 && b < 0 {
        max - 1
    } else {
        a.saturating_add_signed(b)
    }
}

#[derive(Debug, PartialEq)]
enum InputEvent {
    EncInc,
    EncDec,
    EncPush,
    BtnUp,
    BtnDn,
    None,
}

#[derive(Debug)]
enum GuiState {
    Idle,
    Settings,
    ChannelSelect(u8),
    ModeSelect(u8),
    ParameterSelect(u8, u8),
    ParameterEdit(u8, u8),
}

fn main() -> Result<(), core::convert::Infallible> {
    unsafe {
        OUTPUTS.get_or_init(|| core::array::from_fn(|_| NoOutput::new().into()));
    }
    let outputs = unsafe { OUTPUTS.get_mut().unwrap() };

    let mut display =
        SimulatorDisplay::<embedded_graphics::pixelcolor::Bgr565>::new(Size::new(128, 128));
    let output_settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("poco_pico", &output_settings);

    let mut state = GuiState::Idle;

    'main_loop: loop {
        window.update(&display);
        let input = match window.events().next() {
            Some(SimulatorEvent::KeyDown {
                keycode,
                keymod,
                repeat,
            }) if keycode == Keycode::Right => InputEvent::EncInc,
            Some(SimulatorEvent::KeyDown {
                keycode,
                keymod,
                repeat,
            }) if keycode == Keycode::Left => InputEvent::EncDec,
            Some(SimulatorEvent::KeyDown {
                keycode,
                keymod,
                repeat,
            }) if keycode == Keycode::Down => InputEvent::EncPush,
            Some(SimulatorEvent::KeyDown {
                keycode,
                keymod,
                repeat,
            }) if keycode == Keycode::Up => InputEvent::BtnUp,
            Some(SimulatorEvent::KeyDown {
                keycode,
                keymod,
                repeat,
            }) if keycode == Keycode::Return => InputEvent::BtnDn,
            Some(SimulatorEvent::KeyDown {
                keycode,
                keymod,
                repeat,
            }) if keycode == Keycode::Q => break 'main_loop,
            Some(_) => InputEvent::None,
            None => {
                thread::sleep(Duration::from_millis(100));
                InputEvent::None
            }
        };

        if input != InputEvent::None {
            println!("DBG: INPUT: {:?}: STATE: {:?}", input, state);
        }

        // Process Inputs
        let new_state = match state {
            GuiState::Settings => GuiState::Idle,
            GuiState::Idle => {
                match input {
                    InputEvent::EncInc | InputEvent::EncDec | InputEvent::EncPush => {
                        GuiState::ChannelSelect(0)
                    }
                    InputEvent::BtnUp => {
                        // Output.play_pause();
                        GuiState::Idle
                    }
                    InputEvent::BtnDn => GuiState::Settings,
                    InputEvent::None => GuiState::Idle,
                }
            }
            GuiState::ChannelSelect(ch) => {
                match input {
                    InputEvent::EncInc => GuiState::ChannelSelect(add_wrap(ch, 1, 8)),
                    InputEvent::EncDec => GuiState::ChannelSelect(add_wrap(ch, -1, 8)),
                    InputEvent::EncPush => GuiState::ModeSelect(ch),
                    InputEvent::BtnUp => {
                        // Ouput.play_pause()
                        GuiState::ChannelSelect(ch)
                    }
                    InputEvent::BtnDn => GuiState::Idle,
                    InputEvent::None => GuiState::ChannelSelect(ch),
                }
            }
            GuiState::ModeSelect(ch) => {
                match input {
                    InputEvent::EncInc => {
                        outputs[ch as usize] = outputs[ch as usize].next();
                        GuiState::ModeSelect(ch)
                    }
                    InputEvent::EncDec => {
                        outputs[ch as usize] = outputs[ch as usize].prev();
                        GuiState::ModeSelect(ch)
                    }
                    InputEvent::EncPush => GuiState::ParameterSelect(ch, 0),
                    InputEvent::BtnUp => {
                        // Ouput.play_pause()
                        GuiState::ChannelSelect(ch)
                    }
                    InputEvent::BtnDn => GuiState::Idle,
                    InputEvent::None => GuiState::ModeSelect(ch),
                }
            }
            GuiState::ParameterSelect(ch, param) => {
                let num_params = outputs[ch as usize].num_parameters() as u8;
                match input {
                    InputEvent::EncInc => {
                        GuiState::ParameterSelect(ch, add_wrap(param, 1, num_params))
                    }
                    InputEvent::EncDec => {
                        GuiState::ParameterSelect(ch, add_wrap(param, -1, num_params))
                    }
                    InputEvent::EncPush => GuiState::ParameterEdit(ch, param),
                    InputEvent::BtnUp => {
                        // Ouput.play_pause()
                        GuiState::ParameterSelect(ch, param)
                    }
                    InputEvent::BtnDn => GuiState::ChannelSelect(ch),
                    InputEvent::None => GuiState::ParameterSelect(ch, param),
                }
            }
            GuiState::ParameterEdit(ch, param) => {
                if let Some((_, parameter)) = outputs[ch as usize].parameter(param as usize) {
                    match input {
                        InputEvent::EncInc => {
                            parameter.next();
                            GuiState::ParameterEdit(ch, param)
                        }
                        InputEvent::EncDec => {
                            parameter.prev();
                            GuiState::ParameterEdit(ch, param)
                        }
                        InputEvent::EncPush => GuiState::ParameterSelect(ch, param),
                        InputEvent::BtnUp => {
                            // Ouput.play_pause()
                            GuiState::ParameterEdit(ch, param)
                        }
                        InputEvent::BtnDn => GuiState::ParameterSelect(ch, param),
                        InputEvent::None => GuiState::ParameterEdit(ch, param),
                    }
                } else {
                    // error?!
                    GuiState::ParameterSelect(ch, param)
                }
            }
        };

        display.clear(BG);

        let main_window = Rectangle::new(Point::new(0, 10), Size::new(128, 80));
        draw_output_state(&mut display, outputs, 0);

        match state {
            GuiState::Idle => draw_idle(&mut display, main_window),
            GuiState::Settings => draw_idle(&mut display, main_window),
            GuiState::ChannelSelect(ch) => {
                FONT_16.render_aligned(
                    format_args!("{}", ch),
                    Point::new(5, 45),
                    u8g2_fonts::types::VerticalPosition::Bottom,
                    u8g2_fonts::types::HorizontalAlignment::Left,
                    u8g2_fonts::types::FontColor::Transparent(BRIGHT),
                    &mut display,
                );
                let window = Rectangle::new(Point::new(32, 10), Size::new(96, 80));
                outputs[ch as usize].draw_configure(&mut display, window)
            }
            GuiState::ModeSelect(ch) => {}
            GuiState::ParameterSelect(ch, param) => {} //channel[ch].parameter(&disp, param, input),
            GuiState::ParameterEdit(ch, param) => {} //channel[ch].get_param(param).edit(&disp, input),
        }

        state = new_state;
        // TODO Swap buffers
    }

    Ok(())
}

fn draw_idle(display: &mut Display, window: Rectangle) {
    let style = PrimitiveStyle::with_stroke(BLUE, 1);
    window.offset(-2).draw_styled(&style, display);
    let text_pos = window.anchor_point(AnchorPoint::CenterLeft) + Point::new(5, 0);

    //Text::new("IDLE SCREEN", text_pos, TEXT_X10).draw(display);
    FONT_10.render_aligned(
        "Idle Screen",
        text_pos,
        u8g2_fonts::types::VerticalPosition::Bottom,
        u8g2_fonts::types::HorizontalAlignment::Left,
        u8g2_fonts::types::FontColor::Transparent(DARK),
        display,
    );
}

fn draw_output_state(display: &mut Display, outputs: &[OutputChannel], active: u8) {
    let output_disp_corners = [
        Point::new(0, 94),
        Point::new(32, 94),
        Point::new(64, 94),
        Point::new(96, 94),
        Point::new(0, 111),
        Point::new(32, 111),
        Point::new(64, 111),
        Point::new(96, 111),
    ];

    // Draw current output state
    for (out, corner) in zip(outputs, output_disp_corners) {
        let style = PrimitiveStyle::with_stroke(DARK, 1);
        let rect = Rectangle::new(corner, Size::new(32, 16));
        out.draw_output(display, rect);

        let r = RoundedRectangle::new(
            Rectangle::new(corner, Size::new(32, 15)),
            CornerRadii::new(Size::new_equal(3)),
        );
        r.draw_styled(&style, display);
    }
}
