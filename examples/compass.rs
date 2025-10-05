//! # E-Compass Example
//!
//! This example demonstrates the LSM303DLHC e-compass module, which combines
//! a 3-axis accelerometer and 3-axis magnetometer. It shows how to read
//! acceleration, magnetic field, and calculate compass heading.
//!
//! ## What This Example Does
//!
//! - Initializes I2C communication with the LSM303DLHC
//! - Reads 3-axis acceleration data (gravity and motion)
//! - Reads 3-axis magnetic field data (Earth's magnetic field)
//! - Calculates compass heading from magnetometer data
//! - Displays temperature from the internal sensor
//!
//! ## Running the Example
//!
//! ```bash
//! cargo run --example compass
//! ```
//!
//! Tilt and rotate the board to see the sensor values change:
//! - Acceleration shows gravity vector and any motion
//! - Magnetic field shows Earth's magnetic field strength
//! - Heading shows compass direction (0-360 degrees)
//!
//! ## Hardware Used
//!
//! - LSM303DLHC e-compass (I2C interface)
//!   - I2C1 peripheral
//!   - SCL: PB6
//!   - SDA: PB9
//!   - I2C address: 0x19 (accelerometer), 0x1E (magnetometer)
//!
//! ## Sensor Capabilities
//!
//! **Accelerometer:**
//! - Measurement ranges: ±2g, ±4g, ±8g, ±16g
//! - 12-bit resolution
//! - Up to 1.344 kHz output data rate
//!
//! **Magnetometer:**
//! - Measurement ranges: ±1.3 to ±8.1 gauss
//! - 12-bit resolution
//! - Up to 220 Hz output data rate
//!
//! ## Understanding the Output
//!
//! **Acceleration (mg = milli-g):**
//! - 1000 mg = 1g = 9.8 m/s²
//! - When stationary: magnitude ≈ 1000 mg (gravity)
//! - X/Y/Z show gravity vector based on board orientation
//!
//! **Magnetic Field (mG = milli-gauss):**
//! - Earth's field: 250-650 mG depending on location
//! - Distorted near magnetic materials or electronics
//!
//! **Heading:**
//! - 0° = North, 90° = East, 180° = South, 270° = West
//! - Only accurate when board is level (use accelerometer for tilt compensation)

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::compass::{AccelScale, LSM303DLHC, MagGain};
use {defmt_rtt as _, panic_probe as _};

/// Main entry point - demonstrates accelerometer and magnetometer usage
///
/// This example shows how to:
/// - Initialize I2C communication with multiple sensors
/// - Read acceleration and magnetic field data
/// - Perform basic sensor fusion (heading calculation)
/// - Handle multi-byte sensor data with proper scaling
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("E-Compass demo - reading LSM303DLHC accelerometer and magnetometer");
    
    // Initialize compass (accelerometer + magnetometer)
    let mut compass = LSM303DLHC::new(
        p.I2C1,
        p.PB6,  // SCL
        p.PB9,  // SDA
    );
    
    // Configure sensor ranges
    // Accelerometer: ±4g range for good sensitivity while allowing some motion
    compass.set_accel_scale(AccelScale::G4);
    // Magnetometer: ±1.9 gauss for Earth's magnetic field
    compass.set_mag_gain(MagGain::Gauss1_9);
    
    info!("Starting compass readings (tilt and rotate the board)");
    
    loop {
        // Read acceleration
        let accel = compass.read_acceleration();
        
        // Read magnetic field
        let mag = compass.read_magnetic_field();
        
        // Calculate heading
        let heading = LSM303DLHC::calculate_heading(&mag);
        
        // Read temperature
        let temp = compass.read_temperature();
        
        // Display acceleration values in milli-g (mg)
        // 1g = 1000mg = Earth's gravity
        info!(
            "Accel - X: {} mg, Y: {} mg, Z: {} mg",
            (accel.x * 1000.0) as i32,  // Left/Right
            (accel.y * 1000.0) as i32,  // Forward/Back
            (accel.z * 1000.0) as i32   // Up/Down
        );
        
        // Display magnetic field in milli-gauss (mG) and calculated heading
        info!(
            "Mag - X: {} mG, Y: {} mG, Z: {} mG | Heading: {}°",
            (mag.x * 1000.0) as i32,    // Magnetic field X
            (mag.y * 1000.0) as i32,    // Magnetic field Y
            (mag.z * 1000.0) as i32,    // Magnetic field Z
            heading as u16              // Compass heading (0-360°)
        );
        
        // Display temperature from internal sensor
        // Temperature has 8 LSB per degree Celsius resolution
        info!("Temperature: {} °C", temp / 8);
        
        // Update rate: 5 Hz (every 200ms)
        // This is sufficient for most compass applications
        Timer::after_millis(200).await;
    }
}
