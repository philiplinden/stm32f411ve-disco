//! Onboard User LEDs
//!
//! The STM32F411E Discovery board has 4 user LEDs:
//! - LD3 (orange) - PD13 (PWM4 CH2)
//! - LD4 (green)  - PD12 (PWM4 CH1)
//! - LD5 (red)    - PD14 (PWM4 CH3)
//! - LD6 (blue)   - PD15 (PWM4 CH4)

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::Peri;

/// All four user LEDs on the discovery board
pub struct Leds<'d> {
    pub ld3_orange: Output<'d>,
    pub ld4_green: Output<'d>,
    pub ld5_red: Output<'d>,
    pub ld6_blue: Output<'d>,
}

impl<'d> Leds<'d> {
    /// Initialize all four user LEDs (off by default)
    pub fn new(
        pd13: Peri<'d, impl embassy_stm32::gpio::Pin>,
        pd12: Peri<'d, impl embassy_stm32::gpio::Pin>,
        pd14: Peri<'d, impl embassy_stm32::gpio::Pin>,
        pd15: Peri<'d, impl embassy_stm32::gpio::Pin>,
    ) -> Self {
        Self {
            ld3_orange: Output::new(pd13, Level::Low, Speed::Low),
            ld4_green: Output::new(pd12, Level::Low, Speed::Low),
            ld5_red: Output::new(pd14, Level::Low, Speed::Low),
            ld6_blue: Output::new(pd15, Level::Low, Speed::Low),
        }
    }

    /// Turn all LEDs off
    pub fn all_off(&mut self) {
        self.ld3_orange.set_low();
        self.ld4_green.set_low();
        self.ld5_red.set_low();
        self.ld6_blue.set_low();
    }

    /// Turn all LEDs on
    pub fn all_on(&mut self) {
        self.ld3_orange.set_high();
        self.ld4_green.set_high();
        self.ld5_red.set_high();
        self.ld6_blue.set_high();
    }
}
