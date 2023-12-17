//!
//! Tasks for getting temperature from devices
//!

use crate::i2c;
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_executor::SendSpawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use lm75::Address;

/// How many measurements store to get average temperature
const PROCESS_TEMPERATURE_BUFFER_SIZE: usize = 10;

/// Interval between temperature measurements
const GET_TEMP_INTERVAL_MS: u64 = 1000;

/// Channel buffer size
const CHANNEL_SIZE: usize = 1;
/// Temperature gotten from sensor goes here
pub static TEMPERATURE_INPUT: Channel<CriticalSectionRawMutex, f32, CHANNEL_SIZE> = Channel::new();

pub type OutputTemperatureChannel = Channel<CriticalSectionRawMutex, f32, CHANNEL_SIZE>;
/// Output channel for processed channel
pub static TEMPERATURE_PROCESSED: OutputTemperatureChannel = Channel::new();

/// Spawn tasks for getting temperature
pub fn spawn_temperature_input(i2c: &'static i2c::I2cProtected, spawner: &SendSpawner) {
    spawner.must_spawn(get_temperature_lm75b(I2cDevice::new(i2c)));
    spawner.must_spawn(get_temperature_ds3231(I2cDevice::new(i2c)));
}

/// Spawn tasks for processing temperature, returns channel that can be used to get results
pub fn spawn_process_temperature(spawner: &SendSpawner) -> &'static OutputTemperatureChannel {
    spawner.must_spawn(process_temperature());

    &TEMPERATURE_PROCESSED
}

#[embassy_executor::task]
pub async fn get_temperature_lm75b(i2c: i2c::I2cShared) {
    let mut lm75b = lm75::Lm75::new(i2c, Address::from(0x48));
    lm75b.enable().unwrap();

    loop {
        if let Ok(temp) = lm75b.read_temperature() {
            defmt::trace!("lm75b: temperature is {}", temp);
            TEMPERATURE_INPUT.send(temp).await;
        }

        Timer::after(Duration::from_millis(GET_TEMP_INTERVAL_MS)).await;
    }
}

#[embassy_executor::task]
pub async fn get_temperature_ds3231(i2c: i2c::I2cShared) {
    let mut ds3231 = ds323x::Ds323x::new_ds3231(i2c);
    ds3231.enable().unwrap();

    loop {
        if let Ok(temp) = ds3231.temperature() {
            defmt::trace!("ds3131: temperature is {}", temp);
            TEMPERATURE_INPUT.send(temp).await;
        }

        Timer::after(Duration::from_millis(GET_TEMP_INTERVAL_MS)).await;
    }
}

#[embassy_executor::task]
pub async fn process_temperature() {
    let mut temperatures: [f32; PROCESS_TEMPERATURE_BUFFER_SIZE] =
        [0.0; PROCESS_TEMPERATURE_BUFFER_SIZE];

    loop {
        let mut avg_temp = 0.0;

        for i in &mut temperatures {
            let temp = TEMPERATURE_INPUT.receive().await;
            *i = temp;
            avg_temp += temp / (PROCESS_TEMPERATURE_BUFFER_SIZE as f32);
        }

        defmt::debug!("Average temperature: {}", avg_temp);
    }
}
