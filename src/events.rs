
//! Event and notification system for radio control and status updates.
//!
//! This module defines the event and notification types used for communication between
//! different parts of the radio application. It provides asynchronous channels for sending
//! and receiving system events (such as user actions or commands) and notifications (such as
//! hardware status updates or responses).
//!
//! # Usage
//!
//! - Use [`event_send`] and [`event_receive`] for sending and receiving system events.
//! - Use [`notify_publisher`] and [`notify_subscriber`] for publishing and subscribing to notifications.
//!
//! The channels are implemented using Embassy's async synchronization primitives.

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::pubsub::{PubSubChannel, Subscriber, Publisher};

use si473x::{Si47xxTuneStatus, Si47xxRevision};

/// Events representing user actions or commands for the radio system.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SystemEvent {
    /// Turn on FM radio.
    RadioFmOn,
    /// Turn on AM radio.
    RadioAmOn,
    /// Turn radio off.
    RadioOff,
    /// Seek up to the next station.
    RadioSeekUp,
    /// Seek down to the previous station.
    RadioSeekDown,
    /// Set radio frequency (in MHz for FM, kHz for AM).
    RadioSetFrequency(f32),
    /// Mute audio output.
    RadioMute,
    /// Unmute audio output.
    RadioUnmute,
    /// Increase volume by one step.
    RadioVolumeUp,
    /// Decrease volume by one step.
    RadioVolumeDown,
    /// Set volume to a specific value.
    RadioVolumeSet(u8),
}

/// Notifications representing status updates or responses from the radio hardware.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SystemNotify {
    /// Current tuning status (frequency, signal, etc).
    TuneStatus(Si47xxTuneStatus),
    /// Hardware revision information.
    RevisionInfo(Si47xxRevision),
    /// FM radio has been turned on.
    RadioFmOn,
    /// AM radio has been turned on.
    RadioAmOn,
    /// Radio has been turned off.
    RadioOff,
    /// Audio output has been muted.
    RadioMute,
    /// Audio output has been unmuted.
    RadioUnmute,
    /// Volume has changed to the given value.
    VolumeChanged(u8),
}


/// Notification channel for broadcasting system notifications.
static NOTIFICATION_CHANNEL: PubSubChannel<ThreadModeRawMutex, SystemNotify, 4, 4, 2> = PubSubChannel::new();
/// Event channel for sending system events.
static EVENT_CHANNEL: Channel<ThreadModeRawMutex, SystemEvent, 1> = Channel::new();


/// Asynchronously send a system event to the event channel.
pub async fn event_send(state: SystemEvent) {
    EVENT_CHANNEL.send(state).await;
}


/// Try to send a system event to the event channel without blocking.
///
/// If the channel is full, the event is dropped.
pub fn event_try_send(state: SystemEvent) {
    EVENT_CHANNEL.try_send(state).ok();
}


/// Asynchronously receive the next system event from the event channel.
pub async fn event_receive() -> SystemEvent {
    EVENT_CHANNEL.receive().await
}


/// Create a new subscriber for system notifications.
///
/// Returns a [`Subscriber`] that can receive notifications published to the notification channel.
pub fn notify_subscriber<'a>() -> Result<Subscriber<'a, ThreadModeRawMutex, SystemNotify, 4, 4, 2>, embassy_sync::pubsub::Error> {
    NOTIFICATION_CHANNEL.subscriber()
}


/// Create a new publisher for system notifications.
///
/// Returns a [`Publisher`] that can send notifications to all subscribers.
pub fn notify_publisher<'a>() -> Result<Publisher<'a, ThreadModeRawMutex, SystemNotify, 4, 4, 2>, embassy_sync::pubsub::Error> {
    NOTIFICATION_CHANNEL.publisher()
}
