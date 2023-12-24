//!
//! Channel definition for passing temperature data.around
//!

use core::{
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel as EmbassyChannel,
};

/// How many elements can be stored in the channel.
pub const CHANNEL_SIZE: usize = 1;

/// Channel for getting temperature data
pub type Channel = EmbassyChannel<CriticalSectionRawMutex, f32, CHANNEL_SIZE>;

/// Holds a temperature with an ID of the channel
pub struct TemperatureWithID(usize, f32);

/// Wrapper around heapless::Vec selects the first channel that receives value
pub struct ChannelSelect<'vec, const SIZE: usize> {
    channels: &'vec mut heapless::Vec<Channel, SIZE>,
}

impl<'vec, const SIZE: usize> Future for ChannelSelect<'vec, SIZE> {
    type Output = TemperatureWithID;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for (id, channel) in self.channels.iter().enumerate() {
            let fut = pin!(channel.receive());
            if let Poll::Ready(temperature) = fut.poll(cx) {
                return Poll::Ready(TemperatureWithID(id, temperature));
            }
        }

        Poll::Pending
    }
}
