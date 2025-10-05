//! # Gyroscope Example
//!
//! This example demonstrates reading 3-axis angular rate data from the L3GD20
//! digital gyroscope over SPI. The gyroscope measures rotation speed around
//! each axis in degrees per second.
//!
//! ## What This Example Does
//!
//! - Initializes SPI communication with the L3GD20 gyroscope
//! - Configures the sensor for ±500 degrees/second range
//! - Continuously reads angular velocity on X, Y, and Z axes
//! - Displays the rotation rates and temperature
//!
//! ## Running the Example
//!
//! ```bash
//! cargo run --example gyro
//! ```
//!
//! Move and rotate the board to see the angular rate values change.
//! The values represent rotation speed in degrees per second:
//! - X-axis: Roll (rotation around the long axis)
//! - Y-axis: Pitch (tilt forward/backward)
//! - Z-axis: Yaw (rotation flat on table)
//!
//! ## Hardware Used
//!
//! - L3GD20 3-axis gyroscope (SPI interface)
//!   - SPI1 peripheral
//!   - SCK: PA5
//!   - MISO: PA6
//!   - MOSI: PA7
//!   - CS: PE3
//!
//! ## Sensor Capabilities
//!
//! - Measurement ranges: ±250, ±500, ±2000 degrees/second
//! - 16-bit resolution
//! - Up to 760 Hz output data rate
//! - Built-in temperature sensor
//!
//! ## Understanding the Output
//!
//! - Positive values indicate clockwise rotation (when looking along the axis)
//! - Negative values indicate counter-clockwise rotation
//! - Values near zero when the board is stationary
//! - Temperature reading helps with calibration and drift compensation

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::gyro::{FullScale, L3GD20};
use {defmt_rtt as _, panic_probe as _};

/// Main entry point - continuously reads gyroscope data
///
/// This example shows how to:
/// - Initialize SPI communication with a sensor
/// - Configure sensor parameters (range, data rate)
/// - Poll for new data availability
/// - Read and interpret 3-axis sensor data
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Gyroscope demo - reading L3GD20 3-axis angular rate");
    
    // Initialize gyroscope
    let mut gyro = L3GD20::new(
        p.SPI1,
        p.PA5,  // SCK
        p.PA6,  // MISO
        p.PA7,  // MOSI
        p.PE3,  // CS
    );
    
    // Configure for ±500 dps range - good balance between range and resolution
    // Other options: Dps250 (more precise), Dps2000 (wider range)
    gyro.set_scale(FullScale::Dps500);
    
    info!("Starting gyroscope readings (move the board to see values change)");
    
    loop {
        // Wait for new data to be available from the sensor
        // The sensor updates at its configured data rate (default 95 Hz)
        while !gyro.data_ready() {
            Timer::after_millis(1).await;
        }
        
        // Read angular rate
        let rate = gyro.read_angular_rate();
        
        // Read temperature
        let temp = gyro.read_temperature();
        
        // Display the angular rates and temperature
        // Values are in degrees per second (dps)
        // Cast to i32 for display to avoid decimal formatting
        info!(
            "Angular rate - X: {} dps, Y: {} dps, Z: {} dps | Temp: {} °C",
            rate.x as i32,  // Roll rate
            rate.y as i32,  // Pitch rate
            rate.z as i32,  // Yaw rate
            temp            // Temperature (raw value, 1 LSB = 1°C)
        );
        
        // Update rate: 10 Hz (every 100ms)
        // Adjust this based on your application needs
        Timer::after_millis(100).await;
    }
}
