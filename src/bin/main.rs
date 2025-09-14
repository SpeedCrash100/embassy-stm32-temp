#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m_rt::entry;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _}; // global logger

use embassy_stm32_temp::bsp;

// mod display;
// mod i2c;
// mod temperature;
// mod work_indicator;

#[entry]
fn entry() -> ! {
    // BSP prepares peripherals and runtime and runs main task in LOWEST priority
    bsp::entry(self::main);
}

#[embassy_executor::task]
async fn main(p: bsp::Peripherals, _runtime: bsp::Runtime) {
    let i2c = p.i2c1();

    let mut lm75 = lm75::Lm75::new_pct2075(i2c, lm75::Address::from(0x48));
    if lm75.enable().is_err() {
        defmt::error!("Failed to enable LM75B sensor");
        panic!("LM75B sensor not found")
    }

    loop {
        let temp = lm75.read_temperature();
        match temp {
            Ok(temp) => {
                defmt::info!("Temperature: {}°C", temp);
            }
            Err(_) => {
                defmt::error!("Failed to read temperature from LM75B sensor");
                break;
            }
        }
        Timer::after_millis(100).await;
    }

    defmt::warn!("exit of main");

    drop(p);
}
