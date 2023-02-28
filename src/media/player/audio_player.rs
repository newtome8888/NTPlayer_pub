use crossbeam::atomic::AtomicCell;
use log::info;
use std::{
    cell::Cell,
    sync::{atomic::Ordering, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};

use super::traits::Player;
use crate::{
    entity::EventMessage,
    global::{AUDIO_BUFFER, AUDIO_SUMMARY, EVENT_CHANNEL, GLOBAL_PTS_MILLIS},
    util::error::{safe_send, SuperError},
};

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

pub struct AudioPlayer {
    /// State of the audio player
    state: Arc<AtomicCell<State>>,
    /// Thread id
    tid: Cell<Option<JoinHandle<()>>>,
}

impl AudioPlayer {
    /// `es`: System event sender
    /// `ts`: Benchmark timestamp sender
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicCell::new(State::Stopped)),
            tid: Cell::new(None),
        }
    }

    /// Set the buffer queue which will be used for audio play
    pub fn start(&mut self) -> Result<(), SuperError> {
        let summary = AUDIO_SUMMARY.read().unwrap();
        if summary.is_none() {
            return Ok(());
        }

        // If the thread is already running, stop it first
        if let Some(_tid) = self.tid.take() {
            if !_tid.is_finished() {
                _tid.join().expect("Join audio thread failed!");
            }
        }

        let summary = summary.as_ref().unwrap();
        info!("Starting audio player, summary: {:?}", summary);

        let state = self.state.clone();
        let sleep_duration = Duration::from_millis(summary.play_interval);

        let tid = thread::spawn({
            state.store(State::Playing);
            move || loop {
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
                        GLOBAL_PTS_MILLIS.store(-1, Ordering::Release);
                        thread::sleep(sleep_duration);
                        continue;
                    }
                    State::SeekFinished => {
                        state.store(State::Playing);
                    },
                }

                // Play audio
                if let Some(frame) = AUDIO_BUFFER.pop() {
                    // Update timestamp of playing
                    GLOBAL_PTS_MILLIS.store(frame.pts_millis, Ordering::Release);
                    safe_send(EVENT_CHANNEL.0.send(EventMessage::RenderAudio(frame)));
                }

                thread::sleep(sleep_duration);
            }
        });

        self.tid.set(Some(tid));

        Ok(())
    }
}

impl Player for AudioPlayer {
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
