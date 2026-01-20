use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

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
