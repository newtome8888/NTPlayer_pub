use crossbeam::atomic::AtomicCell;
use log::info;
// use tracing::{info, debug};
use std::{
    cell::Cell,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    entity::EventMessage,
    global::{EVENT_CHANNEL, VIDEO_BUFFER, VIDEO_SUMMARY},
    util::error::{safe_send, SuperError},
};

use super::traits::Player;

pub struct VideoPlayer {
    /// State of the audio player
    state: Arc<AtomicCell<State>>,
    /// Thread id
    tid: Cell<Option<JoinHandle<()>>>,
}

impl VideoPlayer {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicCell::new(State::Stopped)),
            tid: Cell::new(None),
        }
    }

    pub fn start(&mut self) -> Result<(), SuperError> {
        let summary = VIDEO_SUMMARY.read().unwrap();
        if summary.is_none() {
            return Ok(());
        }

        // If the thread is already running, stop it first
        if let Some(_tid) = self.tid.take() {
            if !_tid.is_finished() {
                _tid.join().expect("Join audio thread failed!");
            }
        }

        let sender = &EVENT_CHANNEL.0;
        let summary = summary.as_ref().unwrap();
        info!("Starting video player, summary: {:?}", summary);
        sender.send(EventMessage::Resize((summary.width, summary.height)))?;

        let state = self.state.clone();
        let sleep_duration = Duration::from_millis(summary.play_interval);
        let tid = thread::spawn({
            move || {
                const MAX_WAIT_COUNT: u8 = 2;
                let mut wait_count: u8 = 0;

                state.store(State::Playing);
                loop {
                    // Check player state
                    match state.load() {
                        // Ready to stop or already stopped, exit thread
                        State::ReadyToStop => {
                            state.store(State::Stopped);
                            break;
                        }
                        State::Stopped => break,
                        // Ready to pause or already paused,
                        // do nothing and continue loop
                        State::ReadyToPause => {
                            state.store(State::Paused);
                            thread::sleep(sleep_duration);
                            continue;
                        }
                        State::Paused => {
                            thread::sleep(sleep_duration);
                            continue;
                        }
                        // Ready to play or already playing, just go on
                        State::ReadyToPlay | State::ReadyToResume => {
                            state.store(State::Playing);
                        }
                        State::Playing => {
                            // go on
                        }
                        State::Seeking => {
                            thread::sleep(sleep_duration);
                            continue;
                        }
                        State::SeekFinished => {
                            state.store(State::Playing);
                        }
                    }

                    // Play video
                    if let Some(frame) = VIDEO_BUFFER.pop() {
                        // Send video data to UI
                        safe_send(sender.send(EventMessage::RenderVideo(frame)));

                        thread::sleep(sleep_duration);
                    }
                }
            }
        });

        self.tid.set(Some(tid));

        Ok(())
    }
}

impl Player for VideoPlayer {
    fn play(&mut self) {
        self.state.store(State::ReadyToPlay);
    }

    fn pause(&mut self) {
        self.state.store(State::ReadyToPause);
    }

    fn resume(&mut self) {
        self.state.store(State::ReadyToResume);
    }

    fn stop(&mut self) {
        self.state.store(State::ReadyToStop);
    }

    fn fast_forward(&mut self) {
        todo!();
    }

    fn fast_rewind(&mut self) {
        todo!();
    }

    fn seeking(&mut self) {
        self.state.store(State::Seeking);
    }

    fn seek_finished(&mut self) {
        self.state.store(State::SeekFinished);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Playing,
    Paused,
    Stopped,
    Seeking,
    SeekFinished,

    ReadyToPlay,
    ReadyToPause,
    ReadyToResume,
    ReadyToStop,
}
