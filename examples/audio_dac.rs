#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::audio::{CS43L22, OutputDevice, Volume};
use {defmt_rtt as _, panic_probe as _};

/// Demonstrate audio DAC with beep tones
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
    dac.set_output(OutputDevice::Auto);
    dac.set_volume(Volume::new(80));
    
    // Power on the DAC
    dac.power_on();
    
    info!("Playing beep sequence...");
    
    loop {
        // Play ascending tones
        info!("Ascending tones");
        for freq in [10, 20, 30, 40, 50] {
            dac.beep(freq, 200);
            Timer::after_millis(100).await;
        }
        
        Timer::after_millis(500).await;
        
        // Play descending tones
        info!("Descending tones");
        for freq in [50, 40, 30, 20, 10] {
            dac.beep(freq, 200);
            Timer::after_millis(100).await;
        }
        
        Timer::after_millis(1000).await;
        
        // Test volume control
        info!("Volume test");
        dac.set_volume(Volume::new(30));
        dac.beep(30, 500);
        
        dac.set_volume(Volume::new(60));
        dac.beep(30, 500);
        
        dac.set_volume(Volume::new(90));
        dac.beep(30, 500);
        
        // Reset volume
        dac.set_volume(Volume::new(80));
        
        Timer::after_millis(2000).await;
    }
}
