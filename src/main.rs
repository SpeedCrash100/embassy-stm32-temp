#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::future::pending;

use core::arch::asm;
use cortex_m_rt::entry;
use embassy_executor::InterruptExecutor;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _}; // global logger

mod work_indicator;

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_MEDIUM: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_LOW: InterruptExecutor = InterruptExecutor::new();

#[embassy_executor::task]
async fn high_priority() {
    loop {
        let f = pending::<()>();
        f.await;
    }
}

#[embassy_executor::task]
async fn med_priority() {
    loop {
        let f = pending::<()>();
        f.await;
    }
}

#[embassy_executor::task]
async fn low_priority() {
    loop {
        let f = pending::<()>();
        f.await;
    }
}

#[interrupt]
#[allow(non_snake_case)]
unsafe fn USART6() {
    work_indicator::set_working_enabled(true);
    EXECUTOR_HIGH.on_interrupt()
}

#[interrupt]
#[allow(non_snake_case)]
unsafe fn USART2() {
    work_indicator::set_working_enabled(true);
    EXECUTOR_MEDIUM.on_interrupt()
}

#[interrupt]
#[allow(non_snake_case)]
unsafe fn I2C3_EV() {
    work_indicator::set_working_enabled(true);
    EXECUTOR_LOW.on_interrupt()
}

#[entry]
fn main() -> ! {
    let p = embassy_stm32::init(Config::default());

    let indicator_led = Output::new(p.PA5, Level::Low, Speed::VeryHigh);
    work_indicator::init_pin(indicator_led);

    interrupt::USART6.set_priority(Priority::P6);
    let spawner = EXECUTOR_HIGH.start(interrupt::USART6);
    spawner.spawn(high_priority()).unwrap();

    interrupt::USART2.set_priority(Priority::P7);
    let spawner = EXECUTOR_MEDIUM.start(interrupt::USART2);
    spawner.spawn(med_priority()).unwrap();

    interrupt::I2C3_EV.set_priority(Priority::P8);
    let spawner = EXECUTOR_LOW.start(interrupt::I2C3_EV);
    spawner.spawn(low_priority()).unwrap();

    loop {
        unsafe {
            work_indicator::set_working_enabled(false);
            asm!("wfe");
        }
    }
}
