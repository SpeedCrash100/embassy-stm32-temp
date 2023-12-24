//!
//! Contains sources of the temperature readings
//!

use super::channel::Channel;
use crate::i2c::I2cProtected;

use embassy_executor::SendSpawner;

mod ds3231;
mod lm75b;

/// Maximum possible number of sources
pub const MAX_SOURCES: usize = 2;

/// Channels for sources to use
static SOURCES_CHANNELS: [Channel; MAX_SOURCES] = [Channel::new(), Channel::new()]; // TODO! user array_map or something

/// Information needed to initialize the sources
pub struct SourcesInitInfo<'spawn> {
    /// Bus where devices will be searched
    pub bus: &'static I2cProtected,

    /// Spawner to create task for getting temperatures from sources
    pub spawner: &'spawn SendSpawner,
}

pub fn init(info: &SourcesInitInfo) -> &'static [Channel] {
    // First sensor
    {
        let channel = &SOURCES_CHANNELS[0];
        lm75b::init(info, channel);
    }

    {
        let channel = &SOURCES_CHANNELS[1];
        ds3231::init(info, channel);
    }

    &SOURCES_CHANNELS
}
