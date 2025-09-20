use core::future::Future;

pub trait TemperatureSensor {
    /// Gets current temperature in Celsius
    fn get_temperature(&self) -> impl Future<Output = f32>;
}
