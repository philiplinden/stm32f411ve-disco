//! # STM32F411VE Discovery Board Support Package
//! 
//! A comprehensive BSP (Board Support Package) for the STM32F411E-DISCO development board,
//! built on the Embassy async embedded framework.
//! 
//! ## Overview
//! 
//! This crate provides async drivers for all major onboard peripherals of the STM32F411E
//! Discovery board, making it easy to get started with embedded Rust development on this
//! popular development platform.
//! 
//! ## Board Features
//! 
//! - **MCU**: STM32F411VE (ARM Cortex-M4F @ 100MHz, 512KB Flash, 128KB RAM)
//! - **LEDs**: 4 user-controllable LEDs (orange, green, red, blue)
//! - **Button**: 1 user button for input
//! - **Sensors**: 
//!   - L3GD20 3-axis digital gyroscope (±250/500/2000 dps)
//!   - LSM303DLHC e-compass (3-axis accelerometer + 3-axis magnetometer)
//! - **Audio**:
//!   - MP45DT02 MEMS microphone with PDM output
//!   - CS43L22 audio DAC with headphone/speaker amplifier
//! - **Connectivity**: USB OTG Full-Speed
//! 
//! ## Quick Start
//! 
//! Add this crate to your `Cargo.toml`:
//! 
//! ```toml
//! [dependencies]
//! stm32f411ve-disco = "0.1"
//! ```
//! 
//! ## Examples
//! 
//! The crate includes several examples demonstrating each peripheral:
//! 
//! ```bash
//! # Basic LED control
//! cargo run --example blinky
//! 
//! # Read sensor data
//! cargo run --example gyro
//! cargo run --example compass
//! 
//! # Audio demonstrations
//! cargo run --example microphone
//! cargo run --example audio_dac
//! ```
//! 
//! ## Module Organization
//! 
//! The BSP is organized into logical modules for each peripheral type:
//! 
//! - **Hardware Control**
//!   - [`leds`] - Control the 4 onboard LEDs
//!   - [`button`] - Read the user button state
//!   - [`microphone`] - Interface with the MEMS microphone
//!   - [`audio`] - Control the audio DAC
//! 
//! - **Sensors**
//!   - [`gyro`] - 3-axis gyroscope driver
//!   - [`compass`] - Combined accelerometer and magnetometer driver
//! 
//! ## Usage Example
//! 
//! ```no_run
//! use embassy_executor::Spawner;
//! use stm32f411ve_disco::leds::Leds;
//! 
//! #[embassy_executor::main]
//! async fn main(_spawner: Spawner) {
//!     let p = embassy_stm32::init(Default::default());
//!     
//!     // Initialize the LEDs
//!     let mut leds = Leds::new(p.PD13, p.PD12, p.PD14, p.PD15);
//!     
//!     // Turn on the green LED
//!     leds.ld4_green.set_high();
//! }
//! ```
//! 
//! ## Known Limitations
//! 
//! - **Audio DAC**: Currently provides I2C control only. Full audio playback requires 
//!   I2S peripheral configuration which is not yet implemented.
//! - **Microphone**: Basic GPIO interface only. Full PDM audio capture requires I2S/SPI
//!   with DMA and decimation filtering.
//! - **USB OTG**: Not yet implemented.
//! 
//! ## Safety and Hardware Access
//! 
//! All drivers in this BSP use Embassy's type-safe peripheral ownership system, ensuring
//! that peripherals cannot be accidentally used from multiple places simultaneously.

#![no_std]

// Onboard hardware
pub mod leds;
pub mod button;
pub mod microphone;  // MP45DT02 MEMS microphone
pub mod audio;       // CS43L22 audio DAC

// Onboard sensors
pub mod gyro;        // L3GD20 3-axis gyroscope
pub mod compass;     // LSM303DLHC e-compass (accelerometer + magnetometer)
