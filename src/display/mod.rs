use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_executor::SendSpawner;

use crate::i2c;
use crate::temperature::OutputTemperatureChannel;

mod drawables;
use drawables::TemperatureText;

pub fn spawn_display_tasks(
    temperature: &'static OutputTemperatureChannel,
    i2c: &'static i2c::I2cProtected,
    spawner: &SendSpawner,
) {
    spawner.must_spawn(draw_current_temperature(temperature, I2cDevice::new(i2c)))
}

#[embassy_executor::task]
async fn draw_current_temperature(
    temperature: &'static OutputTemperatureChannel,
    i2c: i2c::I2cShared,
) {
    use embedded_graphics::mono_font::iso_8859_5::FONT_10X20;
    use embedded_graphics::mono_font::MonoTextStyleBuilder;
    use embedded_graphics::pixelcolor::BinaryColor;
    use embedded_graphics::prelude::*;
    use ssd1306::prelude::*;

    let mut subscriber = temperature
        .subscriber()
        .expect("failed to create subscriber");

    let display_interface = ssd1306::I2CDisplayInterface::new(i2c);
    let mut display = ssd1306::Ssd1306::new(
        display_interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    display.init().expect("failed to init display");

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(BinaryColor::On)
        .build();

    loop {
        let temp = subscriber.next_message_pure().await;
        display.clear(BinaryColor::Off).ok();

        TemperatureText::new(Point { x: 32, y: 32 }, text_style)
            .with_temperature(temp)
            .draw(&mut display)
            .ok();

        display.flush().ok();
    }
}
