//! # Button Input Example
//!
//! This example demonstrates reading user button input and using it to control
//! the onboard LEDs. Each button press cycles through different LED states.
//!
//! ## What This Example Does
//!
//! - Initializes the user button (B1) on PA0
//! - Configures all four user LEDs
//! - Polls the button state with debouncing
//! - Cycles through LED colors on each button press:
//!   1. Orange LED only
//!   2. Green LED only
//!   3. Red LED only
//!   4. Blue LED only
//!   5. All LEDs on
//!   (then repeats)
//!
//! ## Running the Example
//!
//! ```bash
//! cargo run --example button
//! ```
//!
//! Press the blue USER button (B1) on the board to cycle through the LED states.
//!
//! ## Hardware Used
//!
//! - User button (B1) on pin PA0
//! - LD3 (Orange) on PD13
//! - LD4 (Green) on PD12
//! - LD5 (Red) on PD14
//! - LD6 (Blue) on PD15
//!
//! ## Button Behavior
//!
//! The button is configured with a pull-down resistor, meaning:
//! - Released = LOW (0V)
//! - Pressed = HIGH (3.3V)
//!
//! The example includes 50ms debouncing to prevent false triggers from
//! mechanical button bounce.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::{button::Button, leds::Leds};
use {defmt_rtt as _, panic_probe as _};

/// Main entry point - demonstrates button input handling
///
/// This example shows how to:
/// - Read digital input from the user button
/// - Detect button press events (rising edge)
/// - Use button input to control multiple outputs
/// - Implement simple debouncing
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Button demo - polling user button to cycle LEDs");

    let button = Button::new(p.PA0);
    let mut leds = Leds::new(p.PD13, p.PD12, p.PD14, p.PD15);

    // State machine for cycling through LED patterns
    let mut state = 0u8;
    // Track previous button state for edge detection
    let mut last_pressed = false;

    loop {
        let pressed = button.is_pressed();
        
        // Detect rising edge (button just pressed)
        if pressed && !last_pressed {
            info!("Button pressed!");
            
            // Cycle through LED states
            leds.all_off();
            match state {
                0 => {
                    info!("Orange LED");
                    leds.ld3_orange.set_high();
                }
                1 => {
                    info!("Green LED");
                    leds.ld4_green.set_high();
                }
                2 => {
                    info!("Red LED");
                    leds.ld5_red.set_high();
                }
                3 => {
                    info!("Blue LED");
                    leds.ld6_blue.set_high();
                }
                _ => {
                    info!("All LEDs");
                    leds.all_on();
                }
            }
            
            state = (state + 1) % 5;
        }
        
        last_pressed = pressed;
        // Small delay for button debouncing (prevents false triggers)
        Timer::after_millis(50).await;
    }
}
