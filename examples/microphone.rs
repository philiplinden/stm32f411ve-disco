#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::microphone::MP45DT02;
use {defmt_rtt as _, panic_probe as _};

/// Demonstrate microphone recording (simplified)
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Microphone demo - MP45DT02 MEMS microphone");
    
    // Initialize microphone
    let mut mic = MP45DT02::new(p.PC3, p.PB10);
    
    info!("Starting audio capture...");
    
    // Buffer for audio samples
    let mut audio_buffer = [0i16; 256];
    
    loop {
        // Start recording
        mic.start_recording().await;
        
        // Read samples (simplified - returns dummy data)
        let samples_read = mic.read_samples(&mut audio_buffer).await;
        info!("Read {} audio samples", samples_read);
        
        // Display some sample values
        info!("First 5 samples: {} {} {} {} {}", 
            audio_buffer[0], 
            audio_buffer[1], 
            audio_buffer[2], 
            audio_buffer[3], 
            audio_buffer[4]
        );
        
        // Stop recording
        mic.stop_recording().await;
        
        Timer::after_millis(1000).await;
    }
}
