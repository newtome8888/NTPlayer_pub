use crossbeam::channel::unbounded;

use sdl2::pixels::Color;
use static_init::dynamic;
use std::sync::{
    atomic::{AtomicI16, AtomicI64},
    RwLock,
};

use crate::{
    media::decoder::{
        AudioBuffer, AudioSummary, SubtitleBuffer, SubtitleSummary, VideoBuffer, VideoSummary,
    },
    EventMessage,
};
use crossbeam::channel::{Receiver, Sender};

//
//
//  Normal static variables
//
//

//
// App related
//
pub const APP_NAME: &str = "NT Player";
pub const LOGO_PATH: &str = "./assets/logo.png";
pub const INIT_WIDTH: u32 = 1024;
pub const INIT_HEIGHT: u32 = 768;

//
// Media related
//
/// Forward or rewind amount each time, Unit: milliseconds
pub const FR_STEP: i64 = 10000;
/// Global volume, modify this value will affect to the play volume
pub static VOLUME: AtomicI16 = AtomicI16::new(50);
pub static VOLUME_STEP: i16 = 10;
pub const MAX_VOLUME: i16 = 2000;
pub const VOLUME_BENCHMARK: f32 = 50.0;
/// Global play timestamp, unit milliseconds+
pub static GLOBAL_PTS_MILLIS: AtomicI64 = AtomicI64::new(0);
pub static AUDIO_SUMMARY: RwLock<Option<AudioSummary>> = RwLock::new(None);
pub static VIDEO_SUMMARY: RwLock<Option<VideoSummary>> = RwLock::new(None);
pub static SUBTITLE_SUMMARY: RwLock<Option<SubtitleSummary>> = RwLock::new(None);

pub type EventSender = Sender<EventMessage>;
pub type EventReceiver = Receiver<EventMessage>;

//
//
// Lazy static variables
//
//

#[dynamic]
pub static EVENT_CHANNEL: (EventSender, EventReceiver) = unbounded();
// It's bettrer to give more buffers for audio, 
// becuase humans are more sensitive to sound than video.
// In other words, video frames can be exhausted before audio frames,
// but not vice versa.
#[dynamic]
pub static AUDIO_BUFFER: AudioBuffer = AudioBuffer::new(50);
#[dynamic]
pub static VIDEO_BUFFER: VideoBuffer = VideoBuffer::new(10);
#[dynamic]
pub static SUBTITLE_BUFFER: SubtitleBuffer = SubtitleBuffer::new(5);
