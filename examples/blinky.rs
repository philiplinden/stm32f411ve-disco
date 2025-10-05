#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::leds::Leds;
use {defmt_rtt as _, panic_probe as _};

/// Blink the green LED (LD4) on the STM32F411E Discovery board
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Blinky example - blinking green LED");

    let mut leds = Leds::new(p.PD13, p.PD12, p.PD14, p.PD15);

    loop {
        info!("LED on");
        leds.ld4_green.set_high();
        Timer::after_millis(500).await;

        info!("LED off");
        leds.ld4_green.set_low();
        Timer::after_millis(500).await;
    }
}
