//!
//! Tasks for getting temperature from devices
//!

use crate::i2c;
use embassy_executor::SendSpawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};

mod channel;
use channel::{Channel, ChannelSelect, TemperatureWithID};

mod source;
use source::SourcesInitInfo;

pub type OutputTemperatureChannel = PubSubChannel<CriticalSectionRawMutex, f32, 4, 4, 4>;
/// Output channel for processed channel
static TEMPERATURE_PROCESSED: OutputTemperatureChannel = PubSubChannel::new();

/// Spawn tasks for getting temperature
pub fn spawn_temperature_input(
    i2c: &'static i2c::I2cProtected,
    spawner_sensors: &SendSpawner,
    spawner_process: &SendSpawner,
) -> &'static OutputTemperatureChannel {
    let info = SourcesInitInfo {
        bus: i2c,
        spawner: spawner_sensors,
    };

    let channels = source::init(&info);
    spawner_process.must_spawn(process_temperature(channels));

    &TEMPERATURE_PROCESSED
}

#[embassy_executor::task]
async fn process_temperature(channels: &'static [Channel]) {
    let mut avg_temp_by_source = [0.0_f32; source::MAX_SOURCES];
    let producer = TEMPERATURE_PROCESSED.publisher().unwrap();

    loop {
        let select = ChannelSelect::new(channels);
        let TemperatureWithID(id, temp) = select.await;
        avg_temp_by_source[id] = temp;

        let avg_temp = avg_temp_by_source.iter().sum::<f32>() / avg_temp_by_source.len() as f32;
        producer.publish_immediate(avg_temp);
    }
}
