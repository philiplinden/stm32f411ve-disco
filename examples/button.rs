#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use stm32f411ve_disco::{button::Button, leds::Leds};
use {defmt_rtt as _, panic_probe as _};

/// Toggle LEDs when user button is pressed
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Button demo - polling user button to cycle LEDs");

    let button = Button::new(p.PA0);
    let mut leds = Leds::new(p.PD13, p.PD12, p.PD14, p.PD15);

    let mut state = 0u8;
    let mut last_pressed = false;

    loop {
        let pressed = button.is_pressed();
        
        // Detect rising edge (button just pressed)
        if pressed && !last_pressed {
            info!("Button pressed!");
            
            // Cycle through LED states
            leds.all_off();
            match state {
                0 => {
                    info!("Orange LED");
                    leds.ld3_orange.set_high();
                }
                1 => {
                    info!("Green LED");
                    leds.ld4_green.set_high();
                }
                2 => {
                    info!("Red LED");
                    leds.ld5_red.set_high();
                }
                3 => {
                    info!("Blue LED");
                    leds.ld6_blue.set_high();
                }
                _ => {
                    info!("All LEDs");
                    leds.all_on();
                }
            }
            
            state = (state + 1) % 5;
        }
        
        last_pressed = pressed;
        Timer::after_millis(50).await;  // Debounce
    }
}
