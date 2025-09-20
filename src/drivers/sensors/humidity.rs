use core::future::Future;

pub trait HumiditySensor {
    /// Gets current humidity in % in Celsius
    fn get_humidity(&self) -> impl Future<Output = f32>;
}
