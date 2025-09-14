use core::future::Future;

use embassy_time::Duration;

pub trait TemperatureSensor {
    /// Gets rate between measurements
    fn rate(&self) -> Duration;

    /// Gets current temperature in Celsius
    fn get_temperature(&self) -> impl Future<Output = f32>;
}
