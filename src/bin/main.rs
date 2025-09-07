#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m_rt::entry;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _}; // global logger

use embassy_stm32_temp::board::nucleo_f411re as bsp;

// mod display;
// mod i2c;
// mod temperature;
// mod work_indicator;

#[embassy_executor::task]
async fn high_priority() {
    defmt::info!("high init");
    loop {
        Timer::after_millis(100).await;
        embassy_time::block_for(Duration::from_millis(12));
        defmt::info!("high");
    }
}

#[embassy_executor::task]
async fn med_priority() {
    defmt::info!("med init");
    loop {
        Timer::after_millis(100).await;
        embassy_time::block_for(Duration::from_millis(45));
        defmt::info!("med");
    }
}

#[embassy_executor::task]
async fn low_priority() {
    defmt::info!("low init");
    loop {
        Timer::after_millis(100).await;
        embassy_time::block_for(Duration::from_millis(140));
        defmt::info!("low");
    }
}

#[entry]
fn entry() -> ! {
    bsp::entry(self::main);
}

#[embassy_executor::task]
async fn main(p: bsp::Peripherals, runtime: bsp::Runtime) {
    runtime.must_spawn(0, high_priority());
    runtime.must_spawn(1, med_priority());
    runtime.must_spawn(2, low_priority());

    defmt::warn!("exit of main");

    drop(p);
}
