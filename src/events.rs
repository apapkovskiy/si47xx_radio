use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::pubsub::{PubSubChannel, Subscriber, Publisher};

use si473x::{Si47xxTuneStatus, Si47xxRevision};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SystemEvent {
    RadioFmOn,
    RadioAmOn,
    RadioOff,
    RadioSeekUp,
    RadioSeekDown,
    RadioSetFrequency(f32),
    RadioMute,
    RadioUnmute,
    RadioVolumeUp,
    RadioVolumeDown,
    RadioVolumeSet(u8),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SystemNotify {
    TuneStatus(Si47xxTuneStatus),
    RevisionInfo(Si47xxRevision),
    RadioFmOn,
    RadioAmOn,
    RadioOff,
    RadioMute,
    RadioUnmute,
    VolumeChanged(u8),
}

static NOTIFICATION_CHANNEL: PubSubChannel<ThreadModeRawMutex, SystemNotify, 4, 4, 2> = PubSubChannel::new();
static EVENT_CHANNEL: Channel<ThreadModeRawMutex, SystemEvent, 1> = Channel::new();

pub async fn event_send(state: SystemEvent) {
    EVENT_CHANNEL.send(state).await;
}

pub fn event_try_send(state: SystemEvent) {
    EVENT_CHANNEL.try_send(state).ok();
}

pub async fn event_receive() -> SystemEvent {
    EVENT_CHANNEL.receive().await
}

pub fn notify_subscriber<'a>() -> Result<Subscriber<'a, ThreadModeRawMutex, SystemNotify, 4, 4, 2>, embassy_sync::pubsub::Error> {
    NOTIFICATION_CHANNEL.subscriber()
}

pub fn notify_publisher<'a>() -> Result<Publisher<'a, ThreadModeRawMutex, SystemNotify, 4, 4, 2>, embassy_sync::pubsub::Error> {
    NOTIFICATION_CHANNEL.publisher()
}
