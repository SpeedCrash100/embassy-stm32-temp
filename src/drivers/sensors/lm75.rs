//!
//! LM75 sensor blocking driver
//!
//! LM75 sensor driver requires 2 things to work with:
//! the data which will be used to
//!

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};

use crate::drivers::sensors::temperature::TemperatureSensor;

pub const MEASUREMENT_INTERVAL: Duration = Duration::from_millis(100);

pub struct Shared {
    temperature: Mutex<CriticalSectionRawMutex, f32>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            temperature: Mutex::new(0.0),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Lm75<'a> {
    shared: &'a Shared,
}

impl TemperatureSensor for Lm75<'_> {
    async fn get_temperature(&self) -> f32 {
        *self.shared.temperature.lock().await
    }
}

pub struct Runner<'a> {
    bus: crate::bsp::I2cShared,
    shared: &'a Shared,
}

impl<'a> Runner<'a> {
    pub async fn run(self) -> ! {
        let mut sensor = lm75::Lm75::new_pct2075(self.bus, lm75::Address::from(0x48));

        if sensor.enable().is_err() {
            defmt::error!("Failed to enable LM75B sensor");
        }

        Timer::after_ticks(0).await; // Let others do the job

        loop {
            if let Ok(temp) = sensor.read_temperature() {
                defmt::trace!("lm75b: temperature is {}", temp);
                let mut out_temp = self.shared.temperature.lock().await;
                *out_temp = temp;
            }

            Timer::after(MEASUREMENT_INTERVAL).await;
        }
    }
}

pub fn new<'a>(bus: crate::bsp::I2cShared, data: &'a Shared) -> (Lm75<'a>, Runner<'a>) {
    let runner = Runner { bus, shared: data };
    (Lm75 { shared: data }, runner)
}
