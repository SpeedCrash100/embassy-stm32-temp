use core::cell::RefCell;

use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_stm32::bind_interrupts;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::{ErrorInterruptHandler, EventInterruptHandler, I2c};
use embassy_stm32::peripherals::{I2C1, PB8, PB9};
use embassy_stm32::time::Hertz;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::CriticalSectionMutex;

use static_cell::StaticCell;

pub type I2cHandle = I2c<'static, I2C1, NoDma, NoDma>;
pub type I2cProtected = CriticalSectionMutex<RefCell<I2cHandle>>;
pub type I2cShared = I2cDevice<'static, CriticalSectionRawMutex, I2cHandle>;

bind_interrupts!(struct Irqs {
    I2C1_EV => EventInterruptHandler<I2C1>;
    I2C1_ER => ErrorInterruptHandler<I2C1>;
});

static I2C_HANDLE: StaticCell<I2cProtected> = StaticCell::new();

fn create_i2c_protected(peri: I2C1, scl: PB8, sda: PB9) -> I2cProtected {
    let i2c = I2c::new(
        peri,
        scl,
        sda,
        Irqs,
        NoDma,
        NoDma,
        Hertz(400_000),
        Default::default(),
    );

    let i2c_refcell = RefCell::new(i2c);

    CriticalSectionMutex::new(i2c_refcell)
}

/// Inits the internal I2C bus
///
/// To get created i2c bus use `get_i2c_handle` after usage
pub fn init(peri: I2C1, scl: PB8, sda: PB9) -> &'static mut I2cProtected {
    let bus = create_i2c_protected(peri, scl, sda);

    I2C_HANDLE.init(bus)
}
