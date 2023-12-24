//!
//! DS3231 integrated temperature Sensor
//!

use embassy_time::{Duration, Timer};

use super::SourcesInitInfo;
use crate::{
    i2c::{I2cProtected, I2cShared},
    temperature::channel::Channel,
};

const MEASUREMENT_INTERVAL: Duration = Duration::from_millis(2000);

pub fn init(info: &SourcesInitInfo, channel: &'static Channel) {
    info.spawner.must_spawn(read_temperature(info.bus, channel));
}

#[embassy_executor::task]
async fn read_temperature(bus: &'static I2cProtected, channel: &'static Channel) {
    let mut sensor = ds323x::Ds323x::new_ds3231(I2cShared::new(bus));

    if sensor.enable().is_err() {
        defmt::error!("Failed to enable DS3231 sensor");
    }
    loop {
        if let Ok(temp) = sensor.temperature() {
            defmt::trace!("ds3231: temperature is {}", temp);
            channel.send(temp).await;
        }

        Timer::after(MEASUREMENT_INTERVAL).await;
    }
}
