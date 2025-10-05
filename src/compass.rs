//! LSM303DLHC e-compass driver (3-axis accelerometer + 3-axis magnetometer)
//!
//! The LSM303DLHC is a system-in-package featuring a 3D digital linear acceleration sensor
//! and a 3D digital magnetic sensor. It includes an I2C serial bus interface.
//!
//! ## Features
//! - 3-axis accelerometer: ±2g/±4g/±8g/±16g full scale
//! - 3-axis magnetometer: ±1.3/±1.9/±2.5/±4.0/±4.7/±5.6/±8.1 gauss full scale
//! - 16-bit data output
//! - I2C interface (up to 400 kHz)
//!
//! ## Pin connections on STM32F411E-DISCO:
//! - SCL: PB6
//! - SDA: PB9
//!
//! [Datasheet](https://www.st.com/resource/en/datasheet/lsm303dlhc.pdf)

use defmt::{debug, info};
use embassy_stm32::i2c::{Config as I2cConfig, I2c};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::{DMA1_CH6, DMA1_CH7, I2C1, PB6, PB9};
use embassy_stm32::time::Hertz;
use embassy_stm32::Peripheral;
use embassy_time::{Duration, Timer};

/// I2C addresses
const ACCEL_ADDR: u8 = 0x19; // 0x32 >> 1
const MAG_ADDR: u8 = 0x1E;   // 0x3C >> 1

/// Accelerometer register addresses
#[allow(dead_code)]
mod accel_regs {
    pub const CTRL_REG1_A: u8 = 0x20;
    pub const CTRL_REG2_A: u8 = 0x21;
    pub const CTRL_REG3_A: u8 = 0x22;
    pub const CTRL_REG4_A: u8 = 0x23;
    pub const CTRL_REG5_A: u8 = 0x24;
    pub const CTRL_REG6_A: u8 = 0x25;
    pub const REFERENCE_A: u8 = 0x26;
    pub const STATUS_REG_A: u8 = 0x27;
    pub const OUT_X_L_A: u8 = 0x28;
    pub const OUT_X_H_A: u8 = 0x29;
    pub const OUT_Y_L_A: u8 = 0x2A;
    pub const OUT_Y_H_A: u8 = 0x2B;
    pub const OUT_Z_L_A: u8 = 0x2C;
    pub const OUT_Z_H_A: u8 = 0x2D;
}

/// Magnetometer register addresses
#[allow(dead_code)]
mod mag_regs {
    pub const CRA_REG_M: u8 = 0x00;
    pub const CRB_REG_M: u8 = 0x01;
    pub const MR_REG_M: u8 = 0x02;
    pub const OUT_X_H_M: u8 = 0x03;
    pub const OUT_X_L_M: u8 = 0x04;
    pub const OUT_Z_H_M: u8 = 0x05;
    pub const OUT_Z_L_M: u8 = 0x06;
    pub const OUT_Y_H_M: u8 = 0x07;
    pub const OUT_Y_L_M: u8 = 0x08;
    pub const SR_REG_M: u8 = 0x09;
    pub const IRA_REG_M: u8 = 0x0A;
    pub const IRB_REG_M: u8 = 0x0B;
    pub const IRC_REG_M: u8 = 0x0C;
    pub const TEMP_OUT_H_M: u8 = 0x31;
    pub const TEMP_OUT_L_M: u8 = 0x32;
}

/// Accelerometer full scale selection
#[derive(Debug, Clone, Copy)]
pub enum AccelScale {
    /// ±2g
    G2 = 0x00,
    /// ±4g
    G4 = 0x10,
    /// ±8g
    G8 = 0x20,
    /// ±16g
    G16 = 0x30,
}

impl AccelScale {
    /// Get sensitivity in mg/LSB (millig per LSB)
    fn sensitivity(&self) -> f32 {
        match self {
            AccelScale::G2 => 1.0,
            AccelScale::G4 => 2.0,
            AccelScale::G8 => 4.0,
            AccelScale::G16 => 12.0,
        }
    }
}

/// Magnetometer gain selection
#[derive(Debug, Clone, Copy)]
pub enum MagGain {
    /// ±1.3 gauss
    Gauss1_3 = 0x20,
    /// ±1.9 gauss
    Gauss1_9 = 0x40,
    /// ±2.5 gauss
    Gauss2_5 = 0x60,
    /// ±4.0 gauss
    Gauss4_0 = 0x80,
    /// ±4.7 gauss
    Gauss4_7 = 0xA0,
    /// ±5.6 gauss
    Gauss5_6 = 0xC0,
    /// ±8.1 gauss
    Gauss8_1 = 0xE0,
}

impl MagGain {
    /// Get sensitivity in LSB/gauss
    fn sensitivity_xy(&self) -> f32 {
        match self {
            MagGain::Gauss1_3 => 1100.0,
            MagGain::Gauss1_9 => 855.0,
            MagGain::Gauss2_5 => 670.0,
            MagGain::Gauss4_0 => 450.0,
            MagGain::Gauss4_7 => 400.0,
            MagGain::Gauss5_6 => 330.0,
            MagGain::Gauss8_1 => 230.0,
        }
    }
    
    fn sensitivity_z(&self) -> f32 {
        match self {
            MagGain::Gauss1_3 => 980.0,
            MagGain::Gauss1_9 => 760.0,
            MagGain::Gauss2_5 => 600.0,
            MagGain::Gauss4_0 => 400.0,
            MagGain::Gauss4_7 => 355.0,
            MagGain::Gauss5_6 => 295.0,
            MagGain::Gauss8_1 => 205.0,
        }
    }
}

/// Accelerometer data rate
#[derive(Debug, Clone, Copy)]
pub enum AccelDataRate {
    /// Power-down mode
    PowerDown = 0x00,
    /// 1 Hz
    Hz1 = 0x10,
    /// 10 Hz
    Hz10 = 0x20,
    /// 25 Hz
    Hz25 = 0x30,
    /// 50 Hz
    Hz50 = 0x40,
    /// 100 Hz
    Hz100 = 0x50,
    /// 200 Hz
    Hz200 = 0x60,
    /// 400 Hz
    Hz400 = 0x70,
    /// 1620 Hz (Low power)
    Hz1620LP = 0x80,
    /// 1344 Hz (Normal) / 5376 Hz (Low power)
    Hz1344 = 0x90,
}

/// Magnetometer data rate
#[derive(Debug, Clone, Copy)]
pub enum MagDataRate {
    /// 0.75 Hz
    Hz0_75 = 0x00,
    /// 1.5 Hz
    Hz1_5 = 0x04,
    /// 3 Hz
    Hz3 = 0x08,
    /// 7.5 Hz
    Hz7_5 = 0x0C,
    /// 15 Hz
    Hz15 = 0x10,
    /// 30 Hz
    Hz30 = 0x14,
    /// 75 Hz
    Hz75 = 0x18,
    /// 220 Hz
    Hz220 = 0x1C,
}

/// 3-axis acceleration data
#[derive(Debug, Default, Clone, Copy)]
pub struct Acceleration {
    /// X-axis acceleration in g
    pub x: f32,
    /// Y-axis acceleration in g
    pub y: f32,
    /// Z-axis acceleration in g
    pub z: f32,
}

/// 3-axis magnetic field data
#[derive(Debug, Default, Clone, Copy)]
pub struct MagneticField {
    /// X-axis magnetic field in gauss
    pub x: f32,
    /// Y-axis magnetic field in gauss
    pub y: f32,
    /// Z-axis magnetic field in gauss
    pub z: f32,
}

/// LSM303DLHC e-compass driver
pub struct LSM303DLHC<'a> {
    i2c: I2c<'a>,
    accel_scale: AccelScale,
    mag_gain: MagGain,
}

impl<'a> LSM303DLHC<'a> {
    /// Create a new LSM303DLHC driver instance
    pub async fn new(
        i2c1: impl Peripheral<P = I2C1> + 'a,
        scl: impl Peripheral<P = PB6> + 'a,
        sda: impl Peripheral<P = PB9> + 'a,
        tx_dma: impl Peripheral<P = DMA1_CH6> + 'a,
        rx_dma: impl Peripheral<P = DMA1_CH7> + 'a,
    ) -> Self {
        // Configure I2C for 400 kHz
        let mut config = I2cConfig::default();
        config.scl_pullup = true;
        config.sda_pullup = true;
        
        let i2c = I2c::new(i2c1, scl, sda, Hertz(400_000), config, tx_dma, rx_dma);
        
        let mut compass = Self {
            i2c,
            accel_scale: AccelScale::G2,
            mag_gain: MagGain::Gauss1_3,
        };
        
        // Initialize both sensors
        compass.init().await;
        
        compass
    }
    
    /// Initialize the accelerometer and magnetometer
    async fn init(&mut self) {
        // Initialize accelerometer
        // Normal power mode, 100 Hz, all axes enabled
        self.write_accel_register(accel_regs::CTRL_REG1_A, 0x57).await;
        
        // No high-pass filter
        self.write_accel_register(accel_regs::CTRL_REG2_A, 0x00).await;
        
        // No interrupts
        self.write_accel_register(accel_regs::CTRL_REG3_A, 0x00).await;
        
        // Continuous update, default scale (±2g), high resolution
        self.write_accel_register(accel_regs::CTRL_REG4_A, 0x08).await;
        
        // No FIFO
        self.write_accel_register(accel_regs::CTRL_REG5_A, 0x00).await;
        
        Timer::after(Duration::from_millis(10)).await;
        
        // Initialize magnetometer
        // Temperature enabled, 15 Hz data rate
        self.write_mag_register(mag_regs::CRA_REG_M, 0x90).await;
        
        // Default gain (±1.3 gauss)
        self.write_mag_register(mag_regs::CRB_REG_M, 0x20).await;
        
        // Continuous conversion mode
        self.write_mag_register(mag_regs::MR_REG_M, 0x00).await;
        
        Timer::after(Duration::from_millis(10)).await;
        
        info!("LSM303DLHC initialized");
    }
    
    /// Set accelerometer scale
    pub async fn set_accel_scale(&mut self, scale: AccelScale) {
        self.accel_scale = scale;
        let mut ctrl4 = self.read_accel_register(accel_regs::CTRL_REG4_A).await;
        ctrl4 = (ctrl4 & 0xCF) | (scale as u8);
        self.write_accel_register(accel_regs::CTRL_REG4_A, ctrl4).await;
        debug!("Accelerometer scale set to {:?}", scale);
    }
    
    /// Set accelerometer data rate
    pub async fn set_accel_data_rate(&mut self, rate: AccelDataRate) {
        let mut ctrl1 = self.read_accel_register(accel_regs::CTRL_REG1_A).await;
        ctrl1 = (ctrl1 & 0x0F) | (rate as u8);
        self.write_accel_register(accel_regs::CTRL_REG1_A, ctrl1).await;
        debug!("Accelerometer data rate set to {:?}", rate);
    }
    
    /// Set magnetometer gain
    pub async fn set_mag_gain(&mut self, gain: MagGain) {
        self.mag_gain = gain;
        self.write_mag_register(mag_regs::CRB_REG_M, gain as u8).await;
        debug!("Magnetometer gain set to {:?}", gain);
    }
    
    /// Set magnetometer data rate
    pub async fn set_mag_data_rate(&mut self, rate: MagDataRate) {
        let mut cra = self.read_mag_register(mag_regs::CRA_REG_M).await;
        cra = (cra & 0xE3) | (rate as u8);
        self.write_mag_register(mag_regs::CRA_REG_M, cra).await;
        debug!("Magnetometer data rate set to {:?}", rate);
    }
    
    /// Check if new acceleration data is available
    pub async fn accel_data_ready(&mut self) -> bool {
        let status = self.read_accel_register(accel_regs::STATUS_REG_A).await;
        (status & 0x08) != 0 // ZYXDA bit
    }
    
    /// Check if new magnetic data is available
    pub async fn mag_data_ready(&mut self) -> bool {
        let status = self.read_mag_register(mag_regs::SR_REG_M).await;
        (status & 0x01) != 0 // DRDY bit
    }
    
    /// Read acceleration data
    pub async fn read_acceleration(&mut self) -> Acceleration {
        // Read all 6 bytes
        let mut data = [0u8; 6];
        self.read_accel_burst(accel_regs::OUT_X_L_A | 0x80, &mut data).await;
        
        // Convert to signed 16-bit values (12-bit resolution, left-aligned)
        let raw_x = i16::from_le_bytes([data[0], data[1]]) >> 4;
        let raw_y = i16::from_le_bytes([data[2], data[3]]) >> 4;
        let raw_z = i16::from_le_bytes([data[4], data[5]]) >> 4;
        
        // Convert to g using sensitivity
        let sensitivity = self.accel_scale.sensitivity() / 1000.0;
        
        Acceleration {
            x: raw_x as f32 * sensitivity,
            y: raw_y as f32 * sensitivity,
            z: raw_z as f32 * sensitivity,
        }
    }
    
    /// Read magnetic field data
    pub async fn read_magnetic_field(&mut self) -> MagneticField {
        // Read all 6 bytes
        // Note: Register order is X, Z, Y (not X, Y, Z)
        let mut data = [0u8; 6];
        self.read_mag_burst(mag_regs::OUT_X_H_M, &mut data).await;
        
        // Convert to signed 16-bit values (high byte first for magnetometer)
        let raw_x = i16::from_be_bytes([data[0], data[1]]);
        let raw_z = i16::from_be_bytes([data[2], data[3]]);
        let raw_y = i16::from_be_bytes([data[4], data[5]]);
        
        // Convert to gauss
        let sens_xy = self.mag_gain.sensitivity_xy();
        let sens_z = self.mag_gain.sensitivity_z();
        
        MagneticField {
            x: raw_x as f32 / sens_xy,
            y: raw_y as f32 / sens_xy,
            z: raw_z as f32 / sens_z,
        }
    }
    
    /// Read magnetometer temperature
    pub async fn read_temperature(&mut self) -> i16 {
        let high = self.read_mag_register(mag_regs::TEMP_OUT_H_M).await;
        let low = self.read_mag_register(mag_regs::TEMP_OUT_L_M).await;
        i16::from_be_bytes([high, low]) >> 4 // 12-bit resolution
    }
    
    /// Calculate heading from magnetic field (simple 2D compass)
    pub fn calculate_heading(mag: &MagneticField) -> f32 {
        use micromath::F32Ext;
        
        let heading = mag.y.atan2(mag.x);
        let heading_deg = heading * 180.0 / core::f32::consts::PI;
        
        // Normalize to 0-360 degrees
        if heading_deg < 0.0 {
            heading_deg + 360.0
        } else {
            heading_deg
        }
    }
    
    // Accelerometer register access
    async fn read_accel_register(&mut self, reg: u8) -> u8 {
        let mut buf = [0u8; 1];
        self.i2c.write_read(ACCEL_ADDR, &[reg], &mut buf).await.ok();
        buf[0]
    }
    
    async fn read_accel_burst(&mut self, start_reg: u8, buf: &mut [u8]) {
        self.i2c.write_read(ACCEL_ADDR, &[start_reg], buf).await.ok();
    }
    
    async fn write_accel_register(&mut self, reg: u8, value: u8) {
        self.i2c.write(ACCEL_ADDR, &[reg, value]).await.ok();
    }
    
    // Magnetometer register access
    async fn read_mag_register(&mut self, reg: u8) -> u8 {
        let mut buf = [0u8; 1];
        self.i2c.write_read(MAG_ADDR, &[reg], &mut buf).await.ok();
        buf[0]
    }
    
    async fn read_mag_burst(&mut self, start_reg: u8, buf: &mut [u8]) {
        self.i2c.write_read(MAG_ADDR, &[start_reg], buf).await.ok();
    }
    
    async fn write_mag_register(&mut self, reg: u8, value: u8) {
        self.i2c.write(MAG_ADDR, &[reg, value]).await.ok();
    }
}
