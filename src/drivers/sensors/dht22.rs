use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Delay, Duration, Timer};
use embedded_hal::delay::DelayNs;

use crate::{bsp::DhtSingleWirePin, drivers::sensors::temperature::TemperatureSensor};

pub const MEASUREMENT_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Debug, defmt::Format, thiserror::Error)]
enum Error {
    #[error("Timeout")]
    Timeout,
    #[error("Checksum")]
    Checksum,
}

pub struct Shared {
    temperature: Mutex<CriticalSectionRawMutex, f32>,
    humidity: Mutex<CriticalSectionRawMutex, f32>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            temperature: Mutex::new(0.0),
            humidity: Mutex::new(100.0),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Dht22<'a> {
    shared: &'a Shared,
}

impl TemperatureSensor for Dht22<'_> {
    fn rate(&self) -> Duration {
        MEASUREMENT_INTERVAL
    }

    async fn get_temperature(&self) -> f32 {
        *self.shared.temperature.lock().await
    }
}

pub struct Runner<'a> {
    dht_pin: crate::bsp::DhtSingleWirePin,
    delay: Delay,
    shared: &'a Shared,
}

impl Runner<'_> {
    pub async fn run(mut self) -> ! {
        loop {
            if let Err(err) = self.read().await {
                defmt::error!("DHT22 error: {:?}", err);
            }

            Timer::after(MEASUREMENT_INTERVAL).await;
        }
    }

    async fn read(&mut self) -> Result<(), Error> {
        self.dht_pin.set_low();
        Timer::after_millis(18).await;
        self.dht_pin.set_high();
        self.delay.delay_us(40);

        self.wait_for_high(100)?;
        self.wait_for_low(100)?;

        let humidity_high = self.read_byte()?;
        let humidity_low = self.read_byte()?;
        let temperature_high = self.read_byte()?;
        let temperature_low = self.read_byte()?;
        let checksum = self.read_byte()?;

        let sum = humidity_high
            .wrapping_add(humidity_low)
            .wrapping_add(temperature_high)
            .wrapping_add(temperature_low);
        if sum != checksum {
            return Err(Error::Checksum);
        }

        let humidity_value = ((humidity_high as u16) << 8) | (humidity_low as u16);
        let humidity_percentage = humidity_value as f32 / 10.0;

        let temperature_high_clean = temperature_high & 0x7F; // 0x7F = 0111 1111
        let temperature_value = ((temperature_high_clean as u16) << 8) | (temperature_low as u16);
        let mut temperature = temperature_value as f32 / 10.0;

        self.delay.delay_us(30);

        if temperature_high & 0x80 != 0 {
            temperature = -temperature;
        }

        {
            let mut out_temp = self.shared.temperature.lock().await;
            *out_temp = temperature;
        }

        {
            let mut out_hum = self.shared.humidity.lock().await;
            *out_hum = humidity_percentage;
        }

        Ok(())
    }

    fn wait_for_high(&mut self, mut timeout_us: u32) -> Result<(), Error> {
        while timeout_us > 0 {
            if self.dht_pin.is_high() {
                return Ok(());
            } else {
                self.delay.delay_us(1);
                timeout_us -= 1;
            }
        }
        Err(Error::Timeout)
    }

    fn wait_for_low(&mut self, mut timeout_us: u32) -> Result<(), Error> {
        while timeout_us > 0 {
            if self.dht_pin.is_low() {
                return Ok(());
            } else {
                self.delay.delay_us(1);
                timeout_us -= 1;
            }
        }
        Err(Error::Timeout)
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        let mut byte = 0;

        for n in 0..8 {
            self.wait_for_high(100)?;
            self.delay.delay_us(35);
            let is_bit_1 = self.dht_pin.is_high();
            if is_bit_1 {
                let bit_mask = 1 << (7 - (n % 8));
                byte |= bit_mask;
                self.wait_for_low(100)?;
            }
        }

        Ok(byte)
    }
}

pub fn new<'a>(dht_pin: DhtSingleWirePin, data: &'a Shared) -> (Dht22<'a>, Runner<'a>) {
    let runner = Runner {
        dht_pin,
        shared: data,
        delay: Delay,
    };
    (Dht22 { shared: data }, runner)
}
