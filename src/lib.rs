#![no_std]

// Onboard hardware
pub mod leds;
pub mod button;
// pub mod microphone;  // MP45DT02 MEMS microphone
// pub mod audio;       // CS43L22 audio DAC

// Onboard sensors
pub mod gyro;        // L3GD20 3-axis gyroscope
pub mod compass;     // LSM303DLHC e-compass (accelerometer + magnetometer)
