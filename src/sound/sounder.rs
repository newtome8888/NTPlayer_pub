use sdl2::{
    audio::{AudioCallback, AudioQueue, AudioSpecDesired},
    AudioSubsystem,
};

use crate::{
    media::decoder::{AudioFrame, AudioSummary},
    util::error::SuperError,
};

pub struct Sounder {
    device: AudioQueue<f32>,
}

impl Sounder {
    pub fn new(sys: &AudioSubsystem, summary: &AudioSummary) -> Self {
        let spec = AudioSpecDesired {
            freq: Some(summary.sample_rate),
            channels: Some(summary.channels),
            samples: None,
        };

        let device = sys.open_queue::<f32, _>(None, &spec).unwrap();
        Self { device }
    }

    pub fn play_sound(&self, frame: AudioFrame) -> Result<(), SuperError> {
        self.device.queue_audio(&frame.data)?;
        self.device.resume();

        Ok(())
    }
}

struct S16CallBack {
    data: Vec<i16>,
    pos: usize,
    volume: f32,
}

impl AudioCallback for S16CallBack {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        for dst in out.iter_mut() {
            let mut value = *self.data.get(self.pos).unwrap_or(&0);
            let f32_value = (value as f32) * self.volume;
            if f32_value > i16::MAX as f32 {
                value = i16::MAX;
            } else if f32_value < i16::MIN as f32 {
                value = i16::MIN;
            } else {
                value = f32_value as i16;
            }

            // Set volume
            *dst = value;
            // Record current play position
            self.pos += 1;
        }
    }
}
