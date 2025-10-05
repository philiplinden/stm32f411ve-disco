//! L3GD20 3-axis gyroscope driver
//!
//! The L3GD20 is a low-power three-axis angular rate sensor with a digital SPI interface.
//! It provides 16-bit rate value for each axis.
//!
//! ## Features
//! - 3-axis angular rate sensor
//! - ±250/±500/±2000 dps full scale
//! - 16-bit rate value data output
//! - SPI digital output interface (up to 10 MHz)
//!
//! ## Pin connections on STM32F411E-DISCO:
//! - CS: PE3
//! - SCK: PA5
//! - MISO: PA6
//! - MOSI: PA7
//!
//! [Datasheet](https://www.st.com/resource/en/datasheet/l3gd20.pdf)

use defmt::{debug, info};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{spi, Peri};
use embassy_time::{Duration, Timer};

/// L3GD20 register addresses
#[allow(dead_code)]
mod regs {
    pub const WHO_AM_I: u8 = 0x0F;
    pub const CTRL_REG1: u8 = 0x20;
    pub const CTRL_REG2: u8 = 0x21;
    pub const CTRL_REG3: u8 = 0x22;
    pub const CTRL_REG4: u8 = 0x23;
    pub const CTRL_REG5: u8 = 0x24;
    pub const REFERENCE: u8 = 0x25;
    pub const OUT_TEMP: u8 = 0x26;
    pub const STATUS_REG: u8 = 0x27;
    pub const OUT_X_L: u8 = 0x28;
    pub const OUT_X_H: u8 = 0x29;
    pub const OUT_Y_L: u8 = 0x2A;
    pub const OUT_Y_H: u8 = 0x2B;
    pub const OUT_Z_L: u8 = 0x2C;
    pub const OUT_Z_H: u8 = 0x2D;
}

/// Full scale selection
#[derive(Debug, Clone, Copy)]
pub enum FullScale {
    /// ±250 degrees per second
    Dps250 = 0x00,
    /// ±500 degrees per second
    Dps500 = 0x10,
    /// ±2000 degrees per second
    Dps2000 = 0x20,
}

impl FullScale {
    /// Get sensitivity in mdps/digit (millidegrees per second per digit)
    fn sensitivity(&self) -> f32 {
        match self {
            FullScale::Dps250 => 8.75,
            FullScale::Dps500 => 17.5,
            FullScale::Dps2000 => 70.0,
        }
    }
}

/// Output data rate and bandwidth selection
#[derive(Debug, Clone, Copy)]
pub enum DataRate {
    /// 95 Hz, 12.5 Hz cutoff
    Hz95 = 0x00,
    /// 95 Hz, 25 Hz cutoff
    Hz95_25 = 0x10,
    /// 190 Hz, 12.5 Hz cutoff
    Hz190 = 0x40,
    /// 190 Hz, 25 Hz cutoff
    Hz190_25 = 0x50,
    /// 190 Hz, 50 Hz cutoff
    Hz190_50 = 0x60,
    /// 190 Hz, 70 Hz cutoff
    Hz190_70 = 0x70,
    /// 380 Hz, 20 Hz cutoff
    Hz380 = 0x80,
    /// 380 Hz, 25 Hz cutoff
    Hz380_25 = 0x90,
    /// 380 Hz, 50 Hz cutoff
    Hz380_50 = 0xA0,
    /// 380 Hz, 100 Hz cutoff
    Hz380_100 = 0xB0,
    /// 760 Hz, 30 Hz cutoff
    Hz760 = 0xC0,
    /// 760 Hz, 35 Hz cutoff
    Hz760_35 = 0xD0,
    /// 760 Hz, 50 Hz cutoff
    Hz760_50 = 0xE0,
    /// 760 Hz, 100 Hz cutoff
    Hz760_100 = 0xF0,
}

/// 3-axis angular rate data
#[derive(Debug, Default, Clone, Copy)]
pub struct AngularRate {
    /// X-axis angular rate in degrees per second
    pub x: f32,
    /// Y-axis angular rate in degrees per second
    pub y: f32,
    /// Z-axis angular rate in degrees per second
    pub z: f32,
}

/// L3GD20 gyroscope driver
pub struct L3GD20<'a> {
    spi: Spi<'a, Async>,
    cs: Output<'a>,
    scale: FullScale,
}

impl<'a> L3GD20<'a> {
    /// Create a new L3GD20 driver instance
    pub async fn new<T: spi::Instance>(
        spi1: Peri<'a, T>,
        sck: Peri<'a, impl spi::SckPin<T>>,
        miso: Peri<'a, impl spi::MisoPin<T>>,
        mosi: Peri<'a, impl spi::MosiPin<T>>,
        cs: Peri<'a, impl embassy_stm32::gpio::Pin>,
        tx_dma: Peri<'a, impl spi::TxDma<T>>,
        rx_dma: Peri<'a, impl spi::RxDma<T>>,
    ) -> Self {
        // Configure SPI
        let mut config = Config::default();
        config.frequency = Hertz(10_000_000); // 10MHz max
        config.mode = embassy_stm32::spi::Mode {
            polarity: embassy_stm32::spi::Polarity::IdleHigh,
            phase: embassy_stm32::spi::Phase::CaptureOnSecondTransition,
        };
        
        let spi = Spi::new(spi1, sck, mosi, miso, tx_dma, rx_dma, config);
        let cs = Output::new(cs, Level::High, Speed::VeryHigh);
        
        let mut gyro = Self {
            spi,
            cs,
            scale: FullScale::Dps250,
        };
        
        // Initialize the sensor
        gyro.init().await;
        
        gyro
    }
    
    /// Initialize the gyroscope
    async fn init(&mut self) {
        // Reset sequence
        Timer::after(Duration::from_millis(10)).await;
        
        // Check WHO_AM_I register
        let who_am_i = self.read_register(regs::WHO_AM_I).await;
        info!("L3GD20 WHO_AM_I: {:#x} (expected 0xD4 or 0xD7)", who_am_i);
        
        // Power on and enable all axes
        // PD=1 (normal mode), Zen=1, Yen=1, Xen=1
        // Default data rate 95 Hz
        self.write_register(regs::CTRL_REG1, 0x0F).await;
        
        // Normal mode, no high-pass filter
        self.write_register(regs::CTRL_REG2, 0x00).await;
        
        // No interrupts
        self.write_register(regs::CTRL_REG3, 0x00).await;
        
        // Continuous update, default scale (250 dps)
        self.write_register(regs::CTRL_REG4, 0x00).await;
        
        // No FIFO, no high-pass filter
        self.write_register(regs::CTRL_REG5, 0x00).await;
        
        Timer::after(Duration::from_millis(250)).await;
        info!("L3GD20 initialized");
    }
    
    /// Set the full scale range
    pub async fn set_scale(&mut self, scale: FullScale) {
        self.scale = scale;
        let mut ctrl4 = self.read_register(regs::CTRL_REG4).await;
        ctrl4 = (ctrl4 & 0xCF) | (scale as u8);
        self.write_register(regs::CTRL_REG4, ctrl4).await;
        debug!("L3GD20 scale set to {:?}", scale);
    }
    
    /// Set the output data rate
    pub async fn set_data_rate(&mut self, rate: DataRate) {
        let mut ctrl1 = self.read_register(regs::CTRL_REG1).await;
        ctrl1 = (ctrl1 & 0x0F) | (rate as u8);
        self.write_register(regs::CTRL_REG1, ctrl1).await;
        debug!("L3GD20 data rate set to {:?}", rate);
    }
    
    /// Check if new data is available
    pub async fn data_ready(&mut self) -> bool {
        let status = self.read_register(regs::STATUS_REG).await;
        (status & 0x08) != 0 // ZYXDA bit
    }
    
    /// Read angular rate data from all three axes
    pub async fn read_angular_rate(&mut self) -> AngularRate {
        // Read all 6 bytes in one transaction (auto-increment)
        let mut data = [0u8; 6];
        self.read_burst(regs::OUT_X_L | 0x80, &mut data).await;
        
        // Convert to signed 16-bit values
        let raw_x = i16::from_le_bytes([data[0], data[1]]);
        let raw_y = i16::from_le_bytes([data[2], data[3]]);
        let raw_z = i16::from_le_bytes([data[4], data[5]]);
        
        // Convert to degrees per second using sensitivity
        let sensitivity = self.scale.sensitivity() / 1000.0; // Convert mdps to dps
        
        AngularRate {
            x: raw_x as f32 * sensitivity,
            y: raw_y as f32 * sensitivity,
            z: raw_z as f32 * sensitivity,
        }
    }
    
    /// Read temperature (raw value)
    pub async fn read_temperature(&mut self) -> i8 {
        self.read_register(regs::OUT_TEMP).await as i8
    }
    
    /// Read a single register
    async fn read_register(&mut self, reg: u8) -> u8 {
        let mut buf = [0u8; 1];
        self.cs.set_low();
        
        // Send read command (MSB=1 for read)
        let _ = self.spi.transfer(&mut [reg | 0x80], &mut [0]).await;
        let _ = self.spi.transfer(&mut [0], &mut buf).await;
        
        self.cs.set_high();
        buf[0]
    }
    
    /// Read multiple registers (burst mode)
    async fn read_burst(&mut self, start_reg: u8, buf: &mut [u8]) {
        self.cs.set_low();
        
        // Send read command with auto-increment
        let _ = self.spi.transfer(&mut [start_reg | 0x80], &mut [0]).await;
        
        // Create temporary buffer for reading
        let mut dummy = [0u8; 6]; // Max size we'll need
        let len = buf.len().min(6);
        let _ = self.spi.transfer(&mut dummy[..len], &mut buf[..len]).await;
        
        self.cs.set_high();
    }
    
    /// Write to a single register
    async fn write_register(&mut self, reg: u8, value: u8) {
        self.cs.set_low();
        
        // Send write command (MSB=0 for write)
        let _ = self.spi.transfer(&mut [reg & 0x7F, value], &mut [0, 0]).await;
        
        self.cs.set_high();
    }
}
