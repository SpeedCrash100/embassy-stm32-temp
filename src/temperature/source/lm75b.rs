//!
//! LM75B Temperature Sensor
//!

use embassy_time::{Duration, Timer};

use super::SourcesInitInfo;
use crate::{
    i2c::{I2cProtected, I2cShared},
    temperature::channel::Channel,
};

const MEASUREMENT_INTERVAL: Duration = Duration::from_millis(500);

pub fn init(info: &SourcesInitInfo, channel: &'static Channel) {
    info.spawner.must_spawn(read_temperature(info.bus, channel));
}

#[embassy_executor::task]
async fn read_temperature(bus: &'static I2cProtected, channel: &'static Channel) {
    let mut sensor = lm75::Lm75::new(I2cShared::new(bus), lm75::Address::from(0x48));

    if sensor.enable().is_err() {
        defmt::error!("Failed to enable LM75B sensor");
    }
    loop {
        if let Ok(temp) = sensor.read_temperature() {
            defmt::trace!("lm75b: temperature is {}", temp);
            channel.send(temp).await;
        }

        Timer::after(MEASUREMENT_INTERVAL).await;
    }
}
