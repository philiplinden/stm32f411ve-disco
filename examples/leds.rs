//! # LED Patterns Example
//!
//! This example demonstrates control of all four user LEDs on the Discovery board,
//! showing various patterns and animations that can be created.
//!
//! ## What This Example Does
//!
//! Shows three different LED patterns:
//! 1. **Sequential**: Lights turn on one by one in order
//! 2. **Alternate**: Pairs of LEDs alternate (orange/red vs green/blue)
//! 3. **Flash All**: All LEDs blink together
//!
//! ## Running the Example
//!
//! ```bash
//! cargo run --example leds
//! ```
//!
//! Watch the board as it cycles through the different LED patterns.
//!
//! ## Hardware Used
//!
//! All four user LEDs:
//! - LD3 (Orange) on pin PD13
//! - LD4 (Green) on pin PD12
//! - LD5 (Red) on pin PD14
//! - LD6 (Blue) on pin PD15
//!
//! ## LED Control
//!
//! The LEDs are active-high, meaning:
//! - HIGH (3.3V) = LED ON
//! - LOW (0V) = LED OFF
//!
//! Each LED can be controlled individually or all together using the
//! convenience methods `all_on()` and `all_off()`.
//!
//! ## Pattern Ideas
//!
//! You can modify this example to create your own patterns:
//! - Knight Rider style back-and-forth
//! - Binary counter display
//! - Morse code messages
//! - Status indicators for sensor readings
//! - PWM for brightness control (requires timer configuration)

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::leds::Leds;
use {defmt_rtt as _, panic_probe as _};

/// Main entry point - demonstrates various LED patterns
///
/// This example shows how to:
/// - Control multiple GPIO outputs
/// - Create visual patterns with timing
/// - Use the convenience methods for LED groups
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("LED patterns demo");

    let mut leds = Leds::new(p.PD13, p.PD12, p.PD14, p.PD15);

    loop {
        // Pattern 1: Sequential - "Chase" effect
        // LEDs light up one by one, creating a moving light effect
        info!("Pattern: Sequential chase");
        leds.all_off();
        leds.ld3_orange.set_high();  // First LED
        Timer::after_millis(200).await;
        leds.ld4_green.set_high();   // Second LED (previous stays on)
        Timer::after_millis(200).await;
        leds.ld5_red.set_high();     // Third LED
        Timer::after_millis(200).await;
        leds.ld6_blue.set_high();    // Fourth LED (all now on)
        Timer::after_millis(200).await;
        leds.all_off();              // Clear for next pattern
        Timer::after_millis(500).await;

        // Pattern 2: Alternating pairs
        // Orange/Red alternate with Green/Blue
        info!("Pattern: Alternating pairs");
        for _ in 0..4 {
            // Warm colors (orange + red)
            leds.ld3_orange.set_high();
            leds.ld5_red.set_high();
            Timer::after_millis(200).await;
            leds.all_off();
            
            // Cool colors (green + blue)
            leds.ld4_green.set_high();
            leds.ld6_blue.set_high();
            Timer::after_millis(200).await;
            leds.all_off();
        }
        Timer::after_millis(500).await;

        // Pattern 3: Synchronized flash
        // All LEDs blink together - useful for alerts or notifications
        info!("Pattern: Flash all LEDs");
        for _ in 0..3 {
            leds.all_on();   // All LEDs on simultaneously
            Timer::after_millis(100).await;
            leds.all_off();  // All LEDs off simultaneously
            Timer::after_millis(100).await;
        }
        
        // Pause before repeating all patterns
        Timer::after_millis(1000).await;
    }
}
