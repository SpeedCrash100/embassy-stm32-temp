#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    peripherals::PA5,
    Config,
};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _}; // global logger

#[embassy_executor::task]
async fn indicator(mut pin: Output<'static, PA5>) {
    let interval_ms = 500;

    loop {
        pin.set_high();
        Timer::after(Duration::from_millis(interval_ms)).await;
        pin.set_low();
        Timer::after(Duration::from_millis(interval_ms)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    let indicator_led = Output::new(p.PA5, Level::Low, Speed::Low);
    spawner.must_spawn(indicator(indicator_led));
}
