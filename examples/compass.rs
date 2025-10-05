#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::compass::{AccelScale, LSM303DLHC, MagGain};
use {defmt_rtt as _, panic_probe as _};

/// Read and display accelerometer and magnetometer data
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("E-Compass demo - reading LSM303DLHC accelerometer and magnetometer");
    
    // Initialize compass (accelerometer + magnetometer)
    let mut compass = LSM303DLHC::new(
        p.I2C1,
        p.PB6,  // SCL
        p.PB9,  // SDA
        p.DMA1_CH6,
        p.DMA1_CH7,
    ).await;
    
    // Configure sensors
    compass.set_accel_scale(AccelScale::G4).await;
    compass.set_mag_gain(MagGain::Gauss1_9).await;
    
    info!("Starting compass readings (tilt and rotate the board)");
    
    loop {
        // Read acceleration
        let accel = compass.read_acceleration().await;
        
        // Read magnetic field
        let mag = compass.read_magnetic_field().await;
        
        // Calculate heading
        let heading = LSM303DLHC::calculate_heading(&mag);
        
        // Read temperature
        let temp = compass.read_temperature().await;
        
        // Display values
        info!(
            "Accel - X: {} mg, Y: {} mg, Z: {} mg",
            (accel.x * 1000.0) as i32,
            (accel.y * 1000.0) as i32,
            (accel.z * 1000.0) as i32
        );
        
        info!(
            "Mag - X: {} mG, Y: {} mG, Z: {} mG | Heading: {}°",
            (mag.x * 1000.0) as i32,
            (mag.y * 1000.0) as i32,
            (mag.z * 1000.0) as i32,
            heading as u16
        );
        
        info!("Temperature: {} °C", temp / 8); // Temperature scale is 8 LSB/°C
        
        Timer::after_millis(200).await;
    }
}
