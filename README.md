# STM32F411VE Discovery Board Support Package

Board support package (BSP) for the STM32F411E-DISCO (32F411EDISCOVERY) development board, built on the Embassy async framework.

## Hardware

- **MCU**: STM32F411VE (512KB Flash, 128KB RAM, 100MHz)
- **Architecture**: ARMv7E-M with FPU
- **Target**: `thumbv7em-none-eabihf`
- **Debugger**: ST-Link/V2 (onboard)

**Onboard Features:**
- 4 user LEDs: LD3 (orange/PD13), LD4 (green/PD12), LD5 (red/PD14), LD6 (blue/PD15)
- User button on PA0
- L3GD20 3-axis gyroscope (SPI)
- LSM303DLHC e-compass: 3-axis accelerometer + 3-axis magnetometer (I2C)
- MP45DT02 digital MEMS microphone
- CS43L22 audio DAC with class-D speaker driver
- USB OTG Full-Speed with micro-AB connector

**Documentation:**
- [MCU Datasheet](docs/stm32f411ve.pdf)
- [Discovery Kit User Manual](docs/um1842-discovery-kit-with-stm32f411ve-mcu-stmicroelectronics.pdf)
- [L3GD20 Gyroscope Datasheet](docs/l3gd20.pdf)
- [LSM303DLHC E-Compass Datasheet](docs/lsm303dlhc.pdf)
- [MP45DT02 Microphone Datasheet](docs/mp45dt02.pdf)

## Setup

```sh
# Add Rust target
rustup target add thumbv7em-none-eabihf

# Install probe-rs (if not already installed)
cargo install probe-rs-tools --locked
```

## Quick Start

With your STM32F411E Discovery board connected via USB:

```sh
# Test the board connection
probe-rs list

# Run the blinky example to verify everything works
cargo run --example blinky
```

## Building & Flashing

```sh
# Basic Hardware Examples
cargo run --example blinky      # Blink green LED (LD4)
cargo run --example leds        # LED patterns demo - all 4 LEDs
cargo run --example button      # Press button to cycle through LEDs

# Sensor Examples
cargo run --example gyro        # Read gyroscope - rotate the board!
cargo run --example compass      # Read accelerometer/magnetometer

# Audio Examples
cargo run --example microphone  # MEMS microphone demo
cargo run --example audio_dac   # Generate beep tones

# Build without flashing
cargo build --release

# Flash with additional output
cargo run --example blinky --release -- --log-level debug
```

## BSP Modules

### Hardware Control
- **`leds`** - 4 user LEDs (LD3-LD6) with individual and group control
- **`button`** - User button (PA0) with polling support
- **`microphone`** - MP45DT02 MEMS microphone with PDM interface
- **`audio`** - CS43L22 audio DAC with speaker/headphone output and beep generation

### Sensors
- **`gyro`** - L3GD20 3-axis gyroscope with SPI interface
  - ±250/±500/±2000 dps full scale
  - Temperature sensor
  - Configurable data rates
- **`compass`** - LSM303DLHC e-compass with I2C interface
  - 3-axis accelerometer (±2g/±4g/±8g/±16g)
  - 3-axis magnetometer (±1.3 to ±8.1 gauss)
  - Heading calculation
  - Temperature sensor

## Examples

### Basic Hardware
- **`blinky`** - Simple LED blink to verify board setup
- **`leds`** - Demonstrate all LED patterns and animations
- **`button`** - Button-controlled LED cycling

### Sensors
- **`gyro`** - Read and display 3-axis angular rate data
- **`compass`** - Read accelerometer, magnetometer, and calculate heading

### Audio
- **`microphone`** - Capture audio from MEMS microphone (simplified demo)
- **`audio_dac`** - Generate beep tones and control volume

## Project Structure

- `src/` - BSP library modules
- `examples/` - Example applications demonstrating BSP features
- `docs/` - Datasheets and reference manuals
- `Embed.toml` - Probe-rs configuration (chip, protocol, etc.)
- `.cargo/config.toml` - Build defaults

## Configuration Files

### `.cargo/config.toml`
Sets default build target and runner command:
- Target: `thumbv7em-none-eabihf`
- Runner: `probe-rs run --chip stm32f411ve --protocol swd --connect-under-reset`

### `Embed.toml`
Additional probe-rs configuration for RTT logging and advanced flashing options.

### `memory.x`
Linker script defining Flash (512KB) and RAM (128KB) layout for STM32F411VE.
