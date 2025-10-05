#![no_std]
#![no_main]

//! External 4-Digit 7-Segment Multiplexed Display Example
//!
//! This is NOT for onboard hardware - it's for connecting an external display.
//! 
//! Hardware: Common anode 4-digit display
//! - Segment pins (cathodes): PB0-PB7 (A,B,C,D,E,F,G,DP) through 330Ω resistors
//! - Digit pins (anodes): PC0-PC3 (D1,D2,D3,D4) direct connection
//! - Multiplexing: ~1ms per digit (250Hz refresh rate)

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Segment bit positions
const SEG_A: u8 = 0b0000_0001;
const SEG_B: u8 = 0b0000_0010;
const SEG_C: u8 = 0b0000_0100;
const SEG_D: u8 = 0b0000_1000;
const SEG_E: u8 = 0b0001_0000;
const SEG_F: u8 = 0b0010_0000;
const SEG_G: u8 = 0b0100_0000;
const SEG_DP: u8 = 0b1000_0000;

/// Digit patterns for 0-9 (before common-anode inversion)
const DIGITS: [u8; 10] = [
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_E | SEG_F,           // 0
    SEG_B | SEG_C,                                           // 1
    SEG_A | SEG_B | SEG_D | SEG_E | SEG_G,                   // 2
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_G,                   // 3
    SEG_B | SEG_C | SEG_F | SEG_G,                           // 4
    SEG_A | SEG_C | SEG_D | SEG_F | SEG_G,                   // 5
    SEG_A | SEG_C | SEG_D | SEG_E | SEG_F | SEG_G,           // 6
    SEG_A | SEG_B | SEG_C,                                   // 7
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_E | SEG_F | SEG_G,   // 8
    SEG_A | SEG_B | SEG_C | SEG_D | SEG_F | SEG_G,           // 9
];

/// 4-digit multiplexed 7-segment display
struct FourDigitDisplay<'a> {
    // Segment outputs (PB0-PB7)
    seg_a: Output<'a>,
    seg_b: Output<'a>,
    seg_c: Output<'a>,
    seg_d: Output<'a>,
    seg_e: Output<'a>,
    seg_f: Output<'a>,
    seg_g: Output<'a>,
    seg_dp: Output<'a>,
    // Digit select outputs (PC0-PC3)
    digit_1: Output<'a>,
    digit_2: Output<'a>,
    digit_3: Output<'a>,
    digit_4: Output<'a>,
    // Display buffer (4 digits)
    buffer: [u8; 4],
    // Current digit being displayed (0-3)
    current_digit: usize,
}

impl<'a> FourDigitDisplay<'a> {
    /// Display a 4-digit number (0-9999)
    fn set_number(&mut self, num: u16) {
        let num = num.min(9999);
        self.buffer[0] = ((num / 1000) % 10) as u8;
        self.buffer[1] = ((num / 100) % 10) as u8;
        self.buffer[2] = ((num / 10) % 10) as u8;
        self.buffer[3] = (num % 10) as u8;
    }

    /// Multiplex step - updates one digit
    fn multiplex_step(&mut self) {
        // Turn off all digits first
        self.digit_1.set_low();
        self.digit_2.set_low();
        self.digit_3.set_low();
        self.digit_4.set_low();

        // Get pattern for current digit (inverted for common anode)
        let pattern = !DIGITS[self.buffer[self.current_digit] as usize];

        // Set segment outputs
        self.seg_a.set_level(Level::from((pattern & SEG_A) != 0));
        self.seg_b.set_level(Level::from((pattern & SEG_B) != 0));
        self.seg_c.set_level(Level::from((pattern & SEG_C) != 0));
        self.seg_d.set_level(Level::from((pattern & SEG_D) != 0));
        self.seg_e.set_level(Level::from((pattern & SEG_E) != 0));
        self.seg_f.set_level(Level::from((pattern & SEG_F) != 0));
        self.seg_g.set_level(Level::from((pattern & SEG_G) != 0));
        self.seg_dp.set_level(Level::from((pattern & SEG_DP) != 0));

        // Turn on current digit
        match self.current_digit {
            0 => self.digit_1.set_high(),
            1 => self.digit_2.set_high(),
            2 => self.digit_3.set_high(),
            3 => self.digit_4.set_high(),
            _ => {}
        }

        self.current_digit = (self.current_digit + 1) % 4;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("External 7-segment display example");

    let mut display = FourDigitDisplay {
        seg_a: Output::new(p.PB0, Level::High, Speed::Low),
        seg_b: Output::new(p.PB1, Level::High, Speed::Low),
        seg_c: Output::new(p.PB2, Level::High, Speed::Low),
        seg_d: Output::new(p.PB3, Level::High, Speed::Low),
        seg_e: Output::new(p.PB4, Level::High, Speed::Low),
        seg_f: Output::new(p.PB5, Level::High, Speed::Low),
        seg_g: Output::new(p.PB6, Level::High, Speed::Low),
        seg_dp: Output::new(p.PB7, Level::High, Speed::Low),
        digit_1: Output::new(p.PC0, Level::Low, Speed::Low),
        digit_2: Output::new(p.PC1, Level::Low, Speed::Low),
        digit_3: Output::new(p.PC2, Level::Low, Speed::Low),
        digit_4: Output::new(p.PC3, Level::Low, Speed::Low),
        buffer: [0; 4],
        current_digit: 0,
    };

    let mut counter = 0u16;

    loop {
        display.set_number(counter);
        
        // Multiplex for 1 second (250 cycles at 4ms per cycle)
        for _ in 0..250 {
            display.multiplex_step();
            Timer::after_millis(1).await;
        }
        
        counter = (counter + 1) % 10000;
    }
}
