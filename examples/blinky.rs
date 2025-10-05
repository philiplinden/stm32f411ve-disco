//! # Blinky Example
//!
//! This example demonstrates the most basic embedded application - blinking an LED.
//! It's the "Hello World" of embedded programming and verifies that your development
//! environment is properly configured.
//!
//! ## What This Example Does
//!
//! - Initializes the STM32F411VE microcontroller
//! - Configures the green LED (LD4) on pin PD12
//! - Blinks the LED on and off every 500ms
//!
//! ## Running the Example
//!
//! Connect your STM32F411E Discovery board via USB and run:
//! ```bash
//! cargo run --example blinky
//! ```
//!
//! You should see the green LED blinking and debug output in the terminal.
//!
//! ## Hardware Used
//!
//! - LD4 (Green LED) on pin PD12
//!
//! ## Troubleshooting
//!
//! If the LED doesn't blink:
//! - Check that the board is properly connected via USB
//! - Ensure probe-rs is installed: `cargo install probe-rs-tools`
//! - Verify the board is detected: `probe-rs list`

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::leds::Leds;
use {defmt_rtt as _, panic_probe as _};

/// Main entry point for the blinky application
///
/// This async function runs as the main task and continuously
/// toggles the green LED on and off.
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Blinky example - blinking green LED");

    let mut leds = Leds::new(p.PD13, p.PD12, p.PD14, p.PD15);

    loop {
        info!("LED on");
        leds.ld4_green.set_high();
        Timer::after_millis(500).await;

        info!("LED off");
        leds.ld4_green.set_low();
        Timer::after_millis(500).await;
    }
}
