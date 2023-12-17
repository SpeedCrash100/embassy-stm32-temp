//!
//! Tasks for getting temperature from devices
//!

use crate::i2c;
use embassy_time::{Duration, Timer};
use lm75::Address;

#[embassy_executor::task]
pub async fn get_temperature(i2c: i2c::I2cShared) {
    let mut lm75b = lm75::Lm75::new(i2c, Address::from(0x48));
    lm75b.enable().unwrap();

    loop {
        if let Ok(temp) = lm75b.read_temperature() {
            defmt::info!("Temperature is {}", temp);
        }

        Timer::after(Duration::from_hz(1)).await;
    }
}
