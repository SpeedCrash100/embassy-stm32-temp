use core::cell::RefCell;

use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_stm32::{
    i2c::{I2c, Master, SclPin, SdaPin},
    mode::Blocking,
    peripherals::I2C1,
    Peri,
};
use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, CriticalSectionMutex};
use static_cell::StaticCell;

pub type I2cHandle = I2c<'static, Blocking, Master>;
pub type I2cProtected = CriticalSectionMutex<RefCell<I2cHandle>>;
pub type I2cShared = I2cDevice<'static, CriticalSectionRawMutex, I2cHandle>;

static I2C1_HANDLE: StaticCell<I2cProtected> = StaticCell::new();

pub fn init_i2c1(
    i2c1: Peri<'static, I2C1>,
    scl: Peri<'static, impl SclPin<I2C1>>,
    sda: Peri<'static, impl SdaPin<I2C1>>,
) -> &'static I2cProtected {
    let handle = I2c::new_blocking(i2c1, scl, sda, Default::default());
    let protected = CriticalSectionMutex::new(RefCell::new(handle));

    I2C1_HANDLE.init(protected)
}
