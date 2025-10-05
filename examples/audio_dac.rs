//! # Audio DAC Example
//!
//! This example demonstrates the CS43L22 audio DAC control interface.
//! While full audio playback requires I2S configuration (not yet implemented),
//! this shows how to initialize and control the audio chip via I2C.
//!
//! ## What This Example Does
//!
//! - Initializes I2C communication with the CS43L22 audio DAC
//! - Configures audio output (speaker/headphone)
//! - Sets and adjusts volume levels
//! - Attempts to generate beep tones (limited without I2S)
//!
//! ## Running the Example
//!
//! ```bash
//! cargo run --example audio_dac
//! ```
//!
//! **Note:** Due to current limitations, this example demonstrates control
//! functionality but does not produce actual audio output. Full audio requires
//! I2S peripheral configuration.
//!
//! ## Hardware Used
//!
//! - CS43L22 Audio DAC (I2C control interface)
//!   - I2C1 peripheral (shared with compass)
//!   - SCL: PB6
//!   - SDA: PB9
//!   - RESET: PD4
//!   - I2C address: 0x4A
//!
//! ## Current Limitations
//!
//! This example shows I2C control only. Full audio functionality would require:
//! - I2S peripheral setup for audio data streaming
//! - DMA configuration for continuous audio transfer
//! - Audio PLL configuration for precise sample rates
//! - Proper clock configuration (MCLK, SCK, WS)
//!
//! ## CS43L22 Capabilities
//!
//! - 24-bit stereo DAC
//! - Headphone and speaker amplifiers
//! - Programmable volume control
//! - Beep tone generator
//! - Multiple sample rates (8kHz to 96kHz)
//!
//! ## What You Should See
//!
//! The example will:
//! 1. Initialize the audio DAC chip
//! 2. Configure output and volume
//! 3. Attempt beep sequences (no sound without I2S)
//! 4. Demonstrate volume control
//!
//! Debug output will show the control operations being performed.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::audio::{CS43L22, OutputDevice, Volume};
use {defmt_rtt as _, panic_probe as _};

/// Main entry point - demonstrates audio DAC control
///
/// This example shows how to:
/// - Initialize audio hardware via I2C
/// - Configure audio output settings
/// - Control volume programmatically
/// - Attempt tone generation (limited without I2S)
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Audio DAC demo - CS43L22");
    
    // Initialize audio DAC
    let mut dac = CS43L22::new(
        p.I2C1,
        p.PB6,  // SCL (shared with compass)
        p.PB9,  // SDA (shared with compass)
        p.PD4,  // RESET
    );
    
    // Configure audio output
    // Auto mode will detect if headphones are plugged in
    dac.set_output(OutputDevice::Auto);
    // Set initial volume to 80%
    dac.set_volume(Volume::new(80));
    
    // Power on the DAC
    dac.power_on();
    
    info!("Playing beep sequence...");
    
    loop {
        // Attempt to play ascending tones
        // Note: Without I2S setup, these won't produce actual sound
        info!("Attempting ascending tones (I2C control only)");
        for freq in [10, 20, 30, 40, 50] {
            dac.beep(freq, 200);  // Frequency value, 200ms duration
            Timer::after_millis(100).await;
        }
        
        Timer::after_millis(500).await;
        
        // Attempt to play descending tones
        info!("Attempting descending tones (I2C control only)");
        for freq in [50, 40, 30, 20, 10] {
            dac.beep(freq, 200);  // Frequency value, 200ms duration
            Timer::after_millis(100).await;
        }
        
        Timer::after_millis(1000).await;
        
        // Demonstrate volume control
        // These commands work via I2C and would affect actual audio if I2S was configured
        info!("Testing volume control (30%, 60%, 90%)");
        
        dac.set_volume(Volume::new(30));  // Low volume
        dac.beep(30, 500);
        
        dac.set_volume(Volume::new(60));  // Medium volume
        dac.beep(30, 500);
        
        dac.set_volume(Volume::new(90));  // High volume
        dac.beep(30, 500);
        
        // Reset to default volume
        dac.set_volume(Volume::new(80));
        
        // Wait before repeating the sequence
        info!("Waiting 2 seconds before next cycle...");
        Timer::after_millis(2000).await;
    }
}
