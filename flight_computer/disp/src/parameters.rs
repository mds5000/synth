use core::ops::{Add, Deref, Sub};

use crate::Display;

pub trait ConfigParameter {
    fn next(&mut self);
    fn prev(&mut self);

    fn draw_edit(&self, disp: &mut Display);
}

#[derive(Clone)]
pub struct Parameter<T> {
    min: T,
    max: T,
    step: T,
    value: T,
    rollover: bool,
}

impl<T> Parameter<T> {
    pub fn new_saturating(min: T, max: T, step: T, value: T) -> Self {
        Parameter {
            min,
            max,
            value,
            step,
            rollover: false,
        }
    }

    pub fn new_rollover(min: T, max: T, step: T, value: T) -> Self {
        Parameter {
            min,
            max,
            value,
            step,
            rollover: true,
        }
    }
}

impl<T> ConfigParameter for Parameter<T>
where
    T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T>,
{
    fn next(&mut self) {
        let mut new_value = self.value.add(self.step);
        if new_value > self.max {
            new_value = if self.rollover { self.min } else { self.max };
        }

        self.value = new_value;
    }

    fn prev(&mut self) {
        let mut new_value = self.value - self.step;
        if new_value < self.min {
            new_value = if self.rollover { self.max } else { self.min }
        }

        self.value = new_value;
    }

    fn draw_edit(&self, disp: &mut Display) {}
}

impl<T> Deref for Parameter<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
