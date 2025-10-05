#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::gyro::{FullScale, L3GD20};
use {defmt_rtt as _, panic_probe as _};

/// Read and display gyroscope data
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
    
    // Configure for ±500 dps range
    gyro.set_scale(FullScale::Dps500);
    
    info!("Starting gyroscope readings (move the board to see values change)");
    
    loop {
        // Wait for new data
        while !gyro.data_ready() {
            Timer::after_millis(1).await;
        }
        
        // Read angular rate
        let rate = gyro.read_angular_rate();
        
        // Read temperature
        let temp = gyro.read_temperature();
        
        // Display values
        info!(
            "Angular rate - X: {} dps, Y: {} dps, Z: {} dps | Temp: {} °C",
            rate.x as i32,
            rate.y as i32,
            rate.z as i32,
            temp
        );
        
        Timer::after_millis(100).await;
    }
}
