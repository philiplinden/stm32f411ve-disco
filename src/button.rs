//! Onboard User Button
//!
//! The STM32F411E Discovery board has a user button on PA0.
//! The button is active HIGH (pressed = HIGH).

use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::Peri;

/// User button (B1) on PA0
pub struct Button<'d> {
    inner: Input<'d>,
}

impl<'d> Button<'d> {
    /// Initialize the user button with pull-down resistor
    pub fn new(pin: Peri<'d, impl embassy_stm32::gpio::Pin>) -> Self {
        let input = Input::new(pin, Pull::Down);
        Self { inner: input }
    }

    /// Check if button is currently pressed (blocking)
    pub fn is_pressed(&self) -> bool {
        self.inner.is_high()
    }
}
