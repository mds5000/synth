use core::f32::consts::PI;
use heapless::Vec;
use micromath::F32Ext;

use embedded_graphics::{
    geometry::{AnchorPoint, AnchorX, AnchorY},
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    pixelcolor::{raw::RawU16, Bgr565},
    prelude::*,
    primitives::{
        Circle, CornerRadii, Line, Polyline, PrimitiveStyle, Rectangle, RoundedRectangle,
        StyledDrawable,
    },
    text::Text,
};
use embedded_graphics_simulator::{
    sdl2::Keycode, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use u8g2_fonts::{
    fonts::{u8g2_font_bpixel_tr, u8g2_font_bpixeldouble_tr, u8g2_font_logisoso16_tf},
    FontRenderer,
};

pub type Display = SimulatorDisplay<Bgr565>;

pub const FONT_08: FontRenderer = FontRenderer::new::<u8g2_font_bpixel_tr>();
pub const FONT_10: FontRenderer = FontRenderer::new::<u8g2_font_bpixeldouble_tr>();
pub const FONT_16: FontRenderer = FontRenderer::new::<u8g2_font_logisoso16_tf>();
pub const STY_G: PrimitiveStyle<Bgr565> = PrimitiveStyle::with_stroke(Bgr565::new(8, 11, 8), 1);

pub const BRIGHT: Bgr565 = Bgr565::new(0x1A, 0x38, 0x1F);
pub const TAN: Bgr565 = Bgr565::new(0x13, 0x25, 0x10);
pub const DARK: Bgr565 = Bgr565::new(0x10, 0x1E, 0x0C);
pub const BLUE: Bgr565 = Bgr565::new(0x05, 0x0E, 0x13);
pub const BG: Bgr565 = Bgr565::new(0x04, 0x00, 0x09);

pub struct SquareWave {
    points: Vec<Point, 33>,
}

impl SquareWave {
    pub fn new(cycles: u32, duty_cycle: f32, window: &Rectangle) -> Self {
        let cycles = cycles.clamp(1, 16);

        let max_y = window.anchor_y(AnchorY::Top);
        let min_y = window.anchor_y(AnchorY::Bottom);
        let max_x = window.anchor_x(AnchorX::Right);
        let mut x = window.anchor_x(AnchorX::Left);

        let period = (max_y - min_y) / cycles as i32;
        let pulse_width = (period as f32 * duty_cycle).round() as i32;
        let gap_width = period - pulse_width;

        let mut points = Vec::new();
        for _cycle in 0..cycles {
            // Rising Edge
            points.push(Point::new(x, min_y));
            points.push(Point::new(x, max_y));
            x += pulse_width;

            // Falling Edge
            points.push(Point::new(x, max_y));
            points.push(Point::new(x, min_y));
            x += gap_width;
        }
        points.push(Point::new(max_x, min_y));

        SquareWave { points }
    }
}

impl StyledDrawable<PrimitiveStyle<Bgr565>> for SquareWave {
    type Color = Bgr565;
    type Output = ();
    fn draw_styled<D>(
        &self,
        style: &PrimitiveStyle<Bgr565>,
        target: &mut D,
    ) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Polyline::new(&self.points).draw_styled(style, target)
    }
}

const SINE_POINTS: usize = 33;
pub struct SineWave {
    points: Vec<Point, SINE_POINTS>,
}

impl SineWave {
    pub fn new(cycles: f32, window: &Rectangle) -> Self {
        let cycles = cycles.clamp(0.5, 16.0);

        let max_y = window.anchor_y(AnchorY::Top);
        let center = window.anchor_y(AnchorY::Center);
        let min_y = window.anchor_y(AnchorY::Bottom);
        let max_x = window.anchor_x(AnchorX::Right);
        let min_x = window.anchor_x(AnchorX::Left);
        let span = (max_x - min_x);
        let amplitude = (max_y - center) as f32;

        let range = if span < (SINE_POINTS as i32) {
            (min_x..(max_x - 1)).step_by(1)
        } else {
            let step = (max_x - min_x).div_euclid((SINE_POINTS as i32) - 1) as usize;
            (min_x..(max_x - 1)).step_by(step)
        };

        let mut points = Vec::new();
        for x in range {
            let theta: f32 = ((x - min_x) as f32) * (cycles * 2.0 * PI / (span as f32));
            let y = (theta.sin() * amplitude).round() as i32 + center;
            points.push(Point::new(x, y));
        }

        // Add the last point
        let theta: f32 = (cycles * 2.0 * PI);
        let y = (theta.sin() * amplitude).round() as i32 + center;
        points.push(Point::new(max_x, y));

        SineWave { points }
    }
}

impl StyledDrawable<PrimitiveStyle<Bgr565>> for SineWave {
    type Color = Bgr565;
    type Output = ();
    fn draw_styled<D>(
        &self,
        style: &PrimitiveStyle<Bgr565>,
        target: &mut D,
    ) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Polyline::new(&self.points).draw_styled(style, target)
    }
}
