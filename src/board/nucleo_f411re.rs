mod executor;
mod i2c;
mod work_indicator;

use embassy_executor::{InterruptExecutor, SendSpawner, SpawnToken, Spawner};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    interrupt,
    interrupt::{InterruptExt as _, Priority},
    Config,
};
use static_cell::StaticCell;

use executor::Executor;

/// Handle used to shared i2c bus
pub use i2c::I2cShared;

#[non_exhaustive]
pub struct Peripherals {
    i2c1: &'static i2c::I2cProtected,
}

impl Peripherals {
    pub fn i2c1(&self) -> i2c::I2cShared {
        let i2c_ref = self.i2c1;
        i2c::I2cShared::new(i2c_ref)
    }
}

#[non_exhaustive]
#[derive(Clone)]
pub struct Runtime {
    low_spawner: Spawner,
    med_spawner: SendSpawner,
    high_spawner: SendSpawner,
}

impl Runtime {
    /// Spawn task with desired priority: less is higher
    pub fn spawn<S: Send>(
        &self,
        priority: u8,
        token: SpawnToken<S>,
    ) -> Result<(), embassy_executor::SpawnError> {
        let priority = priority.clamp(0, 2);
        if priority == 0 {
            return self.high_spawner.spawn(token);
        }
        if priority == 1 {
            return self.med_spawner.spawn(token);
        }

        return self.low_spawner.spawn(token);
    }

    /// Spawn task with desired priority: less is higher
    pub fn must_spawn<S: Send>(&self, priority: u8, token: SpawnToken<S>) {
        defmt::unwrap!(self.spawn(priority, token));
    }

    /// Gets lowest priority spawner
    pub fn lowest(&self) -> Spawner {
        self.low_spawner
    }

    /// Gets runner with medium priority
    pub fn medium(&self) -> SendSpawner {
        self.med_spawner
    }

    /// Gets runner with highest priority
    ///
    /// # Note
    /// This does not mean its above all interrupts
    pub fn highest(&self) -> SendSpawner {
        self.high_spawner
    }
}

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

fn init() -> Peripherals {
    let p = embassy_stm32::init(Config::default());
    let indicator_led = Output::new(p.PA5, Level::Low, Speed::VeryHigh);
    let static_activity_led = mk_static!(Output<'static>, indicator_led);
    // We are only one calling this, `p` can passed here once only so we only one setting pin
    unsafe { work_indicator::init_pin(static_activity_led) };

    let i2c1 = p.I2C1;
    let scl = p.PB8;
    let sda = p.PB9;
    let i2c1_ref = i2c::init_i2c1(i2c1, scl, sda);

    Peripherals { i2c1: i2c1_ref }
}

pub fn entry<F, S>(main_function: F) -> !
where
    F: FnOnce(Peripherals, Runtime) -> SpawnToken<S>,
{
    let peripherals = self::init();

    static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();
    static EXECUTOR_MEDIUM: InterruptExecutor = InterruptExecutor::new();
    static EXECUTOR_LOW: StaticCell<Executor> = StaticCell::new();

    #[interrupt]
    #[allow(non_snake_case)]
    unsafe fn USART6() {
        work_indicator::set_working_enabled(true);
        EXECUTOR_HIGH.on_interrupt()
    }

    #[interrupt]
    #[allow(non_snake_case)]
    unsafe fn I2C3_EV() {
        work_indicator::set_working_enabled(true);
        EXECUTOR_MEDIUM.on_interrupt()
    }

    interrupt::USART6.set_priority(Priority::P6);
    let spawner_high = EXECUTOR_HIGH.start(interrupt::USART6);
    interrupt::I2C3_EV.set_priority(Priority::P7);
    let spawner_med = EXECUTOR_MEDIUM.start(interrupt::I2C3_EV);

    let executor = EXECUTOR_LOW.init(Executor::new());
    executor.run(|spawner| {
        let runtime = Runtime {
            low_spawner: spawner.clone(),
            med_spawner: spawner_med,
            high_spawner: spawner_high,
        };
        let token = main_function(peripherals, runtime);
        spawner.must_spawn(token);
    });
}
