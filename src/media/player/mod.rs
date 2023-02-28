pub mod audio_player;
pub mod traits;
pub mod vedio_player;

use crate::util::error::SuperError;

use self::{audio_player::AudioPlayer, traits::Player, vedio_player::VideoPlayer};

pub struct MediaPlayer {
    audio_player: AudioPlayer,
    video_player: VideoPlayer,
}

impl MediaPlayer {
    pub fn new() -> Self {
        Self {
            audio_player: AudioPlayer::new(),
            video_player: VideoPlayer::new(),
        }
    }

    pub fn start(&mut self)-> Result<(), SuperError>{
        let audio_player = &mut self.audio_player;
        let video_player = &mut self.video_player;

        audio_player.start()?;
        video_player.start()?;

        Ok(())
    }
}

impl Player for MediaPlayer {
    fn play(&mut self) {
        self.audio_player.play();
        self.video_player.play();
    }

    fn pause(&mut self) {
        self.audio_player.pause();
        self.video_player.pause();
    }

    fn resume(&mut self) {
        self.audio_player.resume();
        self.video_player.resume();
    }

    fn stop(&mut self) {
        self.audio_player.stop();
        self.video_player.stop();
    }

    fn fast_forward(&mut self) {
        self.audio_player.fast_forward();
        self.video_player.fast_forward();
    }

    fn fast_rewind(&mut self) {
        self.audio_player.fast_rewind();
        self.video_player.fast_rewind();
    }

    fn seeking(&mut self) {
        self.audio_player.seeking();
        self.video_player.seeking();
    }

    fn seek_finished(&mut self) {
        self.audio_player.seek_finished();
        self.video_player.seek_finished();
    }
}

enum PlayerState {
    NONE,
    PLAYING,
    SEEKING,
    PAUSING,
    STOPPED,
}