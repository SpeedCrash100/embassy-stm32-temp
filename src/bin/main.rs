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

#[entry]
fn entry() -> ! {
    // BSP prepares peripherals and runtime and runs main task in LOWEST priority
    bsp::entry(self::main);
}

#[embassy_executor::task]
async fn main(p: bsp::Peripherals, runtime: bsp::Runtime) {
    let lm75b_shared = mk_static!(
        embassy_stm32_temp::drivers::sensors::lm75::Shared,
        embassy_stm32_temp::drivers::sensors::lm75::new_sensor_data()
    );

    let (first_sensor, first_sensor_runner) =
        embassy_stm32_temp::drivers::sensors::lm75::new(p.i2c1(), lm75b_shared);

    runtime
        .lowest()
        .must_spawn(lm75_temp_task(first_sensor_runner));

    loop {
        let temp = first_sensor.get_temperature().await;
        defmt::info!("lm75b: temperature is {}", temp);
        Timer::after(first_sensor.rate()).await;
    }

    defmt::warn!("exit of main");

    drop(p);
}
