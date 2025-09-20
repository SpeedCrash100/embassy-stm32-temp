#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m_rt::entry;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _}; // global logger

use embassy_stm32_temp::{bsp, drivers::sensors::temperature::TemperatureSensor};

// mod display;
// mod i2c;
// mod temperature;
// mod work_indicator;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[embassy_executor::task]
async fn lm75_temp_task(runner: embassy_stm32_temp::drivers::sensors::lm75::Runner<'static>) {
    runner.run().await;
}

#[embassy_executor::task]
async fn dht22_temp_task(runner: embassy_stm32_temp::drivers::sensors::dht22::Runner<'static>) {
    runner.run().await;
}

#[entry]
fn entry() -> ! {
    // BSP prepares peripherals and runtime and runs main task in LOWEST priority
    bsp::entry(self::main);
}

#[embassy_executor::task]
async fn main(p: bsp::Peripherals, runtime: bsp::Runtime) {
    let lm75b_shared = mk_static!(
        embassy_stm32_temp::drivers::sensors::lm75::Shared,
        embassy_stm32_temp::drivers::sensors::lm75::Shared::new()
    );

    let (first_sensor, first_sensor_runner) =
        embassy_stm32_temp::drivers::sensors::lm75::new(p.i2c1(), lm75b_shared);

    runtime
        .lowest()
        .must_spawn(lm75_temp_task(first_sensor_runner));

    let dht22_shared = mk_static!(
        embassy_stm32_temp::drivers::sensors::dht22::Shared,
        embassy_stm32_temp::drivers::sensors::dht22::Shared::new()
    );
    let (second_sensor, second_sensor_runner) =
        embassy_stm32_temp::drivers::sensors::dht22::new(p.dht_pin, dht22_shared);
    runtime
        .medium()
        .must_spawn(dht22_temp_task(second_sensor_runner));

    let mut temps: [f32; 2] = [0.0; 2];

    loop {
        temps[0] = first_sensor.get_temperature().await;
        temps[1] = second_sensor.get_temperature().await;
        let avg_temp = (temps[0] + temps[1]) / 2.0;

        defmt::info!("temp: 0={} 1={} avg={}", temps[0], temps[1], avg_temp);
        Timer::after(second_sensor.rate()).await;
    }

    drop(p);
}
