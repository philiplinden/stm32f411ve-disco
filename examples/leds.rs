#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::leds::Leds;
use {defmt_rtt as _, panic_probe as _};

/// Demonstrate all 4 user LEDs with various patterns
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("LED patterns demo");

    let mut leds = Leds::new(p.PD13, p.PD12, p.PD14, p.PD15);

    loop {
        // Pattern 1: Sequential
        info!("Pattern: Sequential");
        leds.all_off();
        leds.ld3_orange.set_high();
        Timer::after_millis(200).await;
        leds.ld4_green.set_high();
        Timer::after_millis(200).await;
        leds.ld5_red.set_high();
        Timer::after_millis(200).await;
        leds.ld6_blue.set_high();
        Timer::after_millis(200).await;
        leds.all_off();
        Timer::after_millis(500).await;

        // Pattern 2: Alternate
        info!("Pattern: Alternate");
        for _ in 0..4 {
            leds.ld3_orange.set_high();
            leds.ld5_red.set_high();
            Timer::after_millis(200).await;
            leds.all_off();
            leds.ld4_green.set_high();
            leds.ld6_blue.set_high();
            Timer::after_millis(200).await;
            leds.all_off();
        }
        Timer::after_millis(500).await;

        // Pattern 3: All flash
        info!("Pattern: Flash all");
        for _ in 0..3 {
            leds.all_on();
            Timer::after_millis(100).await;
            leds.all_off();
            Timer::after_millis(100).await;
        }
        Timer::after_millis(1000).await;
    }
}
