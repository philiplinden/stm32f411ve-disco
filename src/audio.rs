//! CS43L22 audio DAC driver
//!
//! The CS43L22 is a low-power stereo DAC with headphone and speaker amplifiers.
//! It uses I2C for control and I2S for audio data.
//!
//! ## Features
//! - Stereo 24-bit DAC
//! - Headphone amplifier
//! - Speaker amplifier
//! - I2C control interface
//! - I2S audio data interface
//!
//! ## Pin connections on STM32F411E-DISCO:
//! - I2C_SCL: PB6 (shared with compass)
//! - I2C_SDA: PB9 (shared with compass)
//! - I2S_MCK: PC7
//! - I2S_SCK: PC10
//! - I2S_SD: PC12
//! - I2S_WS: PA4
//! - RESET: PD4
//!
//! Note: Full implementation requires complex I2S setup and audio processing

use defmt::{debug, info};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{Config as I2cConfig, I2c};
use embassy_stm32::{i2c, Peri};
use embassy_time::Duration;

/// CS43L22 I2C address
const CS43L22_ADDR: u8 = 0x4A; // 0x94 >> 1

/// CS43L22 register addresses
#[allow(dead_code)]
mod regs {
    pub const ID: u8 = 0x01;
    pub const POWER_CTL1: u8 = 0x02;
    pub const POWER_CTL2: u8 = 0x04;
    pub const CLOCKING_CTL: u8 = 0x05;
    pub const INTERFACE_CTL1: u8 = 0x06;
    pub const INTERFACE_CTL2: u8 = 0x07;
    pub const PASSTHROUGH_A: u8 = 0x08;
    pub const PASSTHROUGH_B: u8 = 0x09;
    pub const ANALOG_ZC_SR: u8 = 0x0A;
    pub const PLAYBACK_CTL1: u8 = 0x0D;
    pub const MISC_CTL: u8 = 0x0E;
    pub const PLAYBACK_CTL2: u8 = 0x0F;
    pub const PASSTHROUGH_VOL_A: u8 = 0x14;
    pub const PASSTHROUGH_VOL_B: u8 = 0x15;
    pub const PCM_VOL_A: u8 = 0x1A;
    pub const PCM_VOL_B: u8 = 0x1B;
    pub const BEEP_FREQ_ON_TIME: u8 = 0x1C;
    pub const BEEP_VOL_OFF_TIME: u8 = 0x1D;
    pub const BEEP_TONE_CFG: u8 = 0x1E;
    pub const TONE_CTL: u8 = 0x1F;
    pub const MASTER_VOL_A: u8 = 0x20;
    pub const MASTER_VOL_B: u8 = 0x21;
    pub const HEADPHONE_VOL_A: u8 = 0x22;
    pub const HEADPHONE_VOL_B: u8 = 0x23;
    pub const SPEAKER_VOL_A: u8 = 0x24;
    pub const SPEAKER_VOL_B: u8 = 0x25;
    pub const CHANNEL_MIXER: u8 = 0x26;
    pub const LIMIT_CTL1: u8 = 0x27;
    pub const LIMIT_CTL2: u8 = 0x28;
    pub const LIMIT_ATTACK: u8 = 0x29;
    pub const STATUS: u8 = 0x2E;
    pub const BATTERY_COMP: u8 = 0x2F;
    pub const VP_BATTERY_LEVEL: u8 = 0x30;
    pub const SPEAKER_STATUS: u8 = 0x31;
    pub const CHARGE_PUMP_FREQ: u8 = 0x34;
}

/// Output device selection
#[derive(Debug, Clone, Copy)]
pub enum OutputDevice {
    /// Auto-detect (default)
    Auto = 0,
    /// Speaker output
    Speaker = 1,
    /// Headphone output  
    Headphone = 2,
    /// Both outputs
    Both = 3,
}

/// Volume level (0-100)
#[derive(Debug, Clone, Copy)]
pub struct Volume(pub u8);

impl Volume {
    /// Create a new volume level (clamped to 0-100)
    pub fn new(level: u8) -> Self {
        Self(level.min(100))
    }
    
    /// Convert to DAC register value
    fn to_dac_value(&self) -> u8 {
        // Convert 0-100 to DAC range (0x00-0xFF)
        ((self.0 as u16 * 255) / 100) as u8
    }
}

/// CS43L22 audio DAC driver
pub struct CS43L22<'a> {
    i2c: I2c<'a, embassy_stm32::mode::Blocking, i2c::Master>,
    reset: Output<'a>,
    output: OutputDevice,
    volume: Volume,
}

impl<'a> CS43L22<'a> {
    /// Create a new CS43L22 driver instance
    /// Note: This shares the I2C bus with the compass, so coordination is needed
    pub fn new<T: i2c::Instance>(
        i2c1: Peri<'a, T>,
        scl: Peri<'a, impl i2c::SclPin<T>>,
        sda: Peri<'a, impl i2c::SdaPin<T>>,
        reset: Peri<'a, impl embassy_stm32::gpio::Pin>,
    ) -> Self {
        // Configure I2C for 100 kHz (CS43L22 max)
        let config = I2cConfig::default();
        
        let i2c = I2c::new_blocking(i2c1, scl, sda, config);
        let mut reset = Output::new(reset, Level::Low, Speed::Low);
        
        // Reset the chip - blocking delay
        embassy_time::block_for(Duration::from_millis(10));
        reset.set_high();
        embassy_time::block_for(Duration::from_millis(10));
        
        let mut dac = Self {
            i2c,
            reset,
            output: OutputDevice::Auto,
            volume: Volume::new(70),
        };
        
        // Initialize the DAC
        dac.init();
        
        dac
    }
    
    /// Initialize the audio DAC
    fn init(&mut self) {
        // Read chip ID (should be 0xE0 for CS43L22)
        let chip_id = self.read_register(regs::ID);
        info!("CS43L22 chip ID: {:#x} (expected 0xE0)", chip_id);
        
        // Keep powered down during configuration
        self.write_register(regs::POWER_CTL1, 0x01);
        
        // Configure clocking (auto-detect MCLK)
        self.write_register(regs::CLOCKING_CTL, 0x80);
        
        // Configure I2S interface (slave mode, I2S format, 16-bit)
        self.write_register(regs::INTERFACE_CTL1, 0x04);
        
        // Set initial volume
        let vol = self.volume.to_dac_value();
        self.write_register(regs::MASTER_VOL_A, vol);
        self.write_register(regs::MASTER_VOL_B, vol);
        
        // Configure output path
        self.write_register(regs::ANALOG_ZC_SR, 0x00);
        info!("CS43L22 initialized");
    }
    
    /// Power on the DAC
    pub fn power_on(&mut self) {
        self.write_register(regs::POWER_CTL1, 0x9E);
        embassy_time::block_for(Duration::from_millis(100));
        info!("CS43L22 powered on");
    }
    
    /// Power off the DAC
    pub fn power_off(&mut self) {
        self.write_register(regs::POWER_CTL1, 0x01);
        info!("CS43L22 powered off");
    }
    
    /// Set the output device
    pub fn set_output(&mut self, output: OutputDevice) {
        self.output = output;
        
        let val = match output {
            OutputDevice::Auto => 0x00,
            OutputDevice::Speaker => 0xFA,
            OutputDevice::Headphone => 0xAF,
            OutputDevice::Both => 0xAA,
        };
        
        self.write_register(regs::POWER_CTL2, val);
        debug!("Output device set to {:?}", output);
    }
    
    /// Set the master volume
    pub fn set_volume(&mut self, volume: Volume) {
        self.volume = volume;
        let val = volume.to_dac_value();
        
        self.write_register(regs::MASTER_VOL_A, val);
        self.write_register(regs::MASTER_VOL_B, val);
        debug!("Volume set to {}%", volume.0);
    }
    
    /// Mute the output
    pub fn mute(&mut self) {
        self.write_register(regs::MASTER_VOL_A, 0x00);
        self.write_register(regs::MASTER_VOL_B, 0x00);
    }
    
    /// Unmute the output
    pub fn unmute(&mut self) {
        let val = self.volume.to_dac_value();
        self.write_register(regs::MASTER_VOL_A, val);
        self.write_register(regs::MASTER_VOL_B, val);
    }
    
    /// Play a beep tone
    pub fn beep(&mut self, frequency: u8, duration_ms: u16) {
        // Configure beep frequency and duration
        self.write_register(regs::BEEP_FREQ_ON_TIME, frequency);
        self.write_register(regs::BEEP_VOL_OFF_TIME, 0x06); // Medium volume
        
        // Enable beep
        self.write_register(regs::BEEP_TONE_CFG, 0xC0);
        
        embassy_time::block_for(Duration::from_millis(duration_ms as u64));
        
        // Disable beep
        self.write_register(regs::BEEP_TONE_CFG, 0x00);
    }
    
    /// Read a register
    fn read_register(&mut self, reg: u8) -> u8 {
        let mut buf = [0u8; 1];
        self.i2c.blocking_write_read(CS43L22_ADDR, &[reg], &mut buf).ok();
        buf[0]
    }
    
    /// Write to a register
    fn write_register(&mut self, reg: u8, value: u8) {
        self.i2c.blocking_write(CS43L22_ADDR, &[reg, value]).ok();
    }
}
