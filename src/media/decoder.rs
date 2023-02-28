use std::{
    error::Error,
    ffi::CString,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, AtomicI64, AtomicU8, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crossbeam::queue::ArrayQueue;
use log::{debug, error, info, warn};
use rsmpeg::{
    avcodec::{AVCodec, AVCodecContext, AVPacket},
    avformat::AVFormatContextInput,
    ffi::{
        av_seek_frame, AVFormatContext,
        AVMediaType_AVMEDIA_TYPE_ATTACHMENT as AVMEDIATYPE_AVMEDIA_TYPE_ATTACHMENT,
        AVMediaType_AVMEDIA_TYPE_AUDIO as AVMEDIATYPE_AVMEDIA_TYPE_AUDIO,
        AVMediaType_AVMEDIA_TYPE_DATA as AVMEDIATYPE_AVMEDIA_TYPE_DATA,
        AVMediaType_AVMEDIA_TYPE_NB as AVMEDIATYPE_AVMEDIA_TYPE_NB,
        AVMediaType_AVMEDIA_TYPE_SUBTITLE as AVMEDIATYPE_AVMEDIA_TYPE_SUBTITLE,
        AVMediaType_AVMEDIA_TYPE_VIDEO as AVMEDIATYPE_AVMEDIA_TYPE_VIDEO, AVSEEK_FLAG_FRAME,
    },
};

use crate::{
    entity::EventMessage,
    global::{
        AUDIO_BUFFER, AUDIO_SUMMARY, EVENT_CHANNEL, SUBTITLE_BUFFER, SUBTITLE_SUMMARY,
        VIDEO_BUFFER, VIDEO_SUMMARY,
    },
    util::{error::safe_send, pixel_format::parse_video_frame, sample_format},
};

/// The wait duration if buffer queues are full
const BUFFER_FULL_SLEEP_DURATION: Duration = Duration::from_millis(200);
/// The maximum number of frames that will be dropped after seek
const MAX_SKIP_FRAMES: u8 = 5;

pub struct MediaDecoder {
    stop_flag: Arc<AtomicBool>,
    audio_seek_to: Arc<AtomicI64>,
    video_seek_to: Arc<AtomicI64>,
}

impl MediaDecoder {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let audio_seek_to = Arc::new(AtomicI64::new(-1));
        let video_seek_to = Arc::new(AtomicI64::new(-1));

        let ctx = MediaDecoder::get_media_context(&path)?;
        let streams = Self::get_streams(&ctx);

        Self::start_task(ctx, streams, &stop_flag, &audio_seek_to, &video_seek_to);

        Ok(Self {
            stop_flag,
            video_seek_to: video_seek_to.clone(),
            audio_seek_to: audio_seek_to.clone(),
        })
    }

    /// Seek to the specified position
    /// `position` is the position to seek to, unit: milliseconds
    pub fn seek_to(&mut self, position: i64) {
        let mut position = position;
        if position < 0 {
            position = 0;
        }

        let vr = VIDEO_SUMMARY.read().unwrap();
        let ar = AUDIO_SUMMARY.read().unwrap();

        if let Some(summary) = ar.as_ref() {
            let start = position / 1000 * summary.timebase_inverse as i64;
            self.audio_seek_to.store(start, Ordering::Release);
        }
        if let Some(summary) = vr.as_ref() {
            let start = position / 1000 * summary.timebase_inverse as i64;
            self.video_seek_to.store(start, Ordering::Release);
        }
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    fn start_task(
        ctx: AVFormatContextInput,
        streams: MediaStreams,
        stop_flag: &Arc<AtomicBool>,
        audio_seek_to: &Arc<AtomicI64>,
        video_seek_to: &Arc<AtomicI64>,
    ) {
        let stop_flag = stop_flag.clone();
        let audio_seek_to = audio_seek_to.clone();
        let video_seek_to = video_seek_to.clone();
        let sender = &EVENT_CHANNEL.0;
        thread::spawn({
            move || {
                let mut audio_stream = streams.audio_stream;
                let mut video_stream = streams.video_stream;
                let subtitle_stream = streams.subtitle_stream;
                let data_stream = streams.data_stream;
                let attachment_stream = streams.attachment_stream;
                let nb_stream = streams.nb_stream;

                let mut ctx = ctx;
                // The pointer of AVFormatContext
                let ctx_ptr = ctx.as_mut_ptr();
                // The number of audio frames that have bee dropped after seek
                let mut audio_dropped_frames = u8::MAX;
                // The number of Video frames that have been dropped after seek
                let mut video_dropped_frames = u8::MAX;

                loop {
                    if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }

                    let seeked = if let Some(index) = &video_stream.index {
                        // If current media is stream, seek with video stream
                        Self::seek_to_stream(ctx_ptr, &video_seek_to, *index)
                    } else if let Some(index) = &audio_stream.index {
                        // Otherwise, seek with audio stream
                        Self::seek_to_stream(ctx_ptr, &audio_seek_to, *index)
                    } else {
                        false
                    };

                    if seeked {
                        // Set the skip number to 0 to indicate some frames need to be dropped later
                        audio_dropped_frames = 0;
                        video_dropped_frames = 0;
                        // Clear old data
                        Self::clear_buffer();
                        // Send seek finish status
                        safe_send(sender.send(EventMessage::SeekFinished));
                    }

                    if AUDIO_BUFFER.is_full() || VIDEO_BUFFER.is_full() || SUBTITLE_BUFFER.is_full()
                    {
                        thread::sleep(BUFFER_FULL_SLEEP_DURATION);
                        continue;
                    }

                    let result = ctx.read_packet().unwrap();
                    match result {
                        Some(packet) => {
                            // Only process the data in correct stream, ignore others
                            let stream_index = Some(packet.stream_index);
                            if stream_index == audio_stream.index {
                                audio_stream.decoder_ctx = audio_stream
                                    .decoder_ctx
                                    .and_then(|dctx| {
                                        let dctx = Self::decode_audio(dctx, &packet, &mut audio_dropped_frames);
                                        Some(dctx)
                                    })
                                    .or_else(|| {
                                        warn!("Audio stream founded but no decoder!");
                                        None
                                    });
                            } else if stream_index == video_stream.index {
                                video_stream.decoder_ctx = video_stream
                                    .decoder_ctx
                                    .and_then(|dctx| {
                                        let dctx = Self::decode_video(dctx, &packet, &mut video_dropped_frames);
                                        Some(dctx)
                                    })
                                    .or_else(|| {
                                        warn!("Video stream founded but no decoder!");
                                        None
                                    });
                            } else if stream_index == subtitle_stream.index {
                                //todo!
                            } else if stream_index == data_stream.index {
                                info!("data stream packet readed");
                            } else if stream_index == nb_stream.index {
                                info!("nb stream packet readed");
                            } else if stream_index == attachment_stream.index {
                                info!("attachment stream packet readed");
                            } else {
                                debug!("unknown type of packet");
                            }
                        }
                        None => {
                            debug!("no more packets, stop decoding");
                            break;
                        }
                    }
                }
            }
        });
    }

    #[inline]
    fn seek_to_stream(
        ctx_ptr: *mut AVFormatContext,
        seek_to: &Arc<AtomicI64>,
        stream_index: i32,
    ) -> bool {
        let position = seek_to.load(Ordering::Acquire);
        if position < 0 {
            return false;
        }

        unsafe { av_seek_frame(ctx_ptr, stream_index, position, AVSEEK_FLAG_FRAME as i32) };

        seek_to.store(-1, Ordering::Release);

        return true;
    }

    // Notice! The context should be returned to return back the ownership
    fn decode_audio(dctx: AVCodecContext, packet: &AVPacket, dropped_frames: &mut u8) -> AVCodecContext {
        let mut dctx = dctx;
        if let Err(err) = dctx.send_packet(Some(packet)) {
            error!("send packet to context error: {}", err);
            return dctx;
        }

        match dctx.receive_frame() {
            Ok(mut frame) => {
                // If value of SKIPPED_FRAMES is smaller than MAX_SKIP_FRAMES,
                // seek action is did a moment before, current frame may not be the correct one,
                // drop some frames to get the correct one.
                // Once dropped frames reached the max number, stop dropping.
                if *dropped_frames < MAX_SKIP_FRAMES {
                    *dropped_frames += 1;
                    return dctx;
                } else {
                    *dropped_frames = u8::MAX;
                }

                let mut audio_frame = sample_format::parse_audio_frame(&mut frame);

                // Push frame to buffer until succeeded
                while let Err(f) = AUDIO_BUFFER.push(audio_frame) {
                    audio_frame = f;
                    thread::sleep(BUFFER_FULL_SLEEP_DURATION);
                }
            }
            Err(err) => {
                error!("{}", err);
            }
        }

        dctx
    }

    // Notice! The context should be returned to return back the ownership
    fn decode_video(dctx: AVCodecContext, packet: &AVPacket, dropped_frames: &mut u8) -> AVCodecContext {
        let mut dctx = dctx;
        if let Err(err) = dctx.send_packet(Some(packet)) {
            error!("send packet to context error: {}", err);
            return dctx;
        }

        match dctx.receive_frame() {
            Ok(frame) => {
                // If value of SKIPPED_FRAMES is smaller than MAX_SKIP_FRAMES,
                // seek action is did a moment before, current frame may not be the correct one,
                // drop some frames to get the correct one.
                // Once dropped frames reached the max number, stop dropping.
                if *dropped_frames < MAX_SKIP_FRAMES {
                    *dropped_frames += 1;
                    return dctx;
                } else {
                    *dropped_frames = u8::MAX;
                }

                let mut vf = parse_video_frame(&frame);
                // Push frame to buffer until succeeded
                while let Err(f) = VIDEO_BUFFER.push(vf) {
                    vf = f;
                    thread::sleep(BUFFER_FULL_SLEEP_DURATION);
                }
            }
            Err(err) => {
                error!("{}", err);
            }
        }

        dctx
    }

    /// Notice! DemuxerWithStreamInfo do not support multiple threads, so you have to create
    /// a new object for every thread which `DemuxerWithStreamInfo` will be used
    pub fn get_media_context(path: &str) -> Result<AVFormatContextInput, Box<dyn Error>> {
        let path = CString::new(path).unwrap();
        let ctx = AVFormatContextInput::open(&path)?;

        Ok(ctx)
    }

    fn get_streams(ctx: &AVFormatContextInput) -> MediaStreams {
        let streams = ctx.streams();

        let mut audio_stream = StreamInfo::default();
        let mut video_stream = StreamInfo::default();
        let mut subtitle_stream = StreamInfo::default();
        let mut data_stream = StreamInfo::default();
        let mut attachment_stream = StreamInfo::default();
        let mut nb_stream = StreamInfo::default();
        let mut unknown_streams = Vec::<StreamInfo>::new();

        for stream in streams {
            if stream.nb_frames <= 0 {
                continue;
            }

            let codecpar = stream.codecpar();
            let codec_type = stream.codecpar().codec_type;

            let mut decoder_name = String::default();
            let decoder_ctx = AVCodec::find_decoder(codecpar.codec_id).and_then(|d| {
                decoder_name = d.name().to_str().unwrap_or("unknown").to_string();
                let mut decoder_ctx = AVCodecContext::new(&d);

                if let Err(err) = decoder_ctx.apply_codecpar(&codecpar) {
                    error!("{}", err);
                }

                if let Err(err) = decoder_ctx.open(None) {
                    error!("{}", err);
                }

                Some(decoder_ctx)
            });
            let stream_info = StreamInfo {
                decoder_ctx,
                index: Some(stream.index),
            };

            let duration = stream.duration as u64;
            let frames = stream.nb_frames as u64;
            let timebase_num = stream.time_base.num as u64;
            let timebase_den = stream.time_base.den as u64;
            let timebase_inverse = timebase_den / timebase_num;
            let duration_millis = 1000 * duration / timebase_inverse;
            let play_interval = duration_millis / frames;

            match codec_type {
                AVMEDIATYPE_AVMEDIA_TYPE_AUDIO => {
                    audio_stream = stream_info;
                    let audio_summary = Some(AudioSummary {
                        decoder_name,
                        duration,
                        duration_millis,
                        frames,
                        timebase_num,
                        timebase_den,
                        timebase_inverse,
                        play_interval,
                        channels: codecpar.channels as u8,
                        channel_layout: codecpar.channel_layout,
                        sample_rate: codecpar.sample_rate,
                        frame_size: codecpar.frame_size,
                    });

                    // Save audio summary to static
                    let mut w = AUDIO_SUMMARY.write().unwrap();
                    *w = audio_summary;
                }
                AVMEDIATYPE_AVMEDIA_TYPE_VIDEO => {
                    video_stream = stream_info;
                    let video_summary = Some(VideoSummary {
                        decoder_name,
                        duration,
                        duration_millis,
                        frames,
                        timebase_num,
                        timebase_den,
                        timebase_inverse,
                        play_interval,
                        width: codecpar.width as u32,
                        height: codecpar.height as u32,
                    });

                    // Save video summary to static
                    let mut w = VIDEO_SUMMARY.write().unwrap();
                    *w = video_summary;
                }
                AVMEDIATYPE_AVMEDIA_TYPE_SUBTITLE => {
                    subtitle_stream = stream_info;
                    let subtitle_summary = Some(SubtitleSummary);

                    // Save subtitle summary to static
                    let mut w = SUBTITLE_SUMMARY.write().unwrap();
                    *w = subtitle_summary;
                }
                AVMEDIATYPE_AVMEDIA_TYPE_ATTACHMENT => {
                    attachment_stream = stream_info;
                }
                AVMEDIATYPE_AVMEDIA_TYPE_DATA => {
                    data_stream = stream_info;
                }
                AVMEDIATYPE_AVMEDIA_TYPE_NB => {
                    nb_stream = stream_info;
                }
                _ => {
                    unknown_streams.push(stream_info);
                }
            }
        }

        MediaStreams {
            audio_stream,
            video_stream,
            subtitle_stream,
            attachment_stream,
            nb_stream,
            data_stream,
            unknown_streams,
        }
    }

    fn clear_buffer() {
        while !AUDIO_BUFFER.is_empty() {
            AUDIO_BUFFER.pop();
        }

        while !VIDEO_BUFFER.is_empty() {
            VIDEO_BUFFER.pop();
        }

        while !SUBTITLE_BUFFER.is_empty() {
            SUBTITLE_BUFFER.pop();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoSummary {
    /// The name of decoder if any
    pub decoder_name: String,
    /// The duration of whole media
    pub duration: u64,
    /// Duration in milliseconds
    pub duration_millis: u64,
    /// Number of frames in media
    pub frames: u64,
    /// Number of timebase
    pub timebase_num: u64,
    /// Denominator of timebase
    pub timebase_den: u64,
    /// The reciprocal of timebase
    pub timebase_inverse: u64,
    /// Play interval with milliseconds
    pub play_interval: u64,
    /// Width of video
    pub width: u32,
    /// Height of video
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioSummary {
    /// The name of decoder if any
    pub decoder_name: String,
    /// The duration of whole media
    pub duration: u64,
    /// Duration in milliseconds
    pub duration_millis: u64,
    /// Number of frames in media
    pub frames: u64,
    /// Number of timebase
    pub timebase_num: u64,
    /// Denominator of timebase
    pub timebase_den: u64,
    /// The reciprocal of timebase
    pub timebase_inverse: u64,
    /// Play interval with milliseconds
    pub play_interval: u64,
    /// Number of channels
    pub channels: u8,
    /// Layout of channel, most of the time but not always can be converted with channels
    pub channel_layout: u64,
    /// Samples per second
    pub sample_rate: i32,
    /// How much samples per frame
    pub frame_size: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleSummary;

struct StreamInfo {
    decoder_ctx: Option<AVCodecContext>,
    index: Option<i32>,
}

impl Default for StreamInfo {
    fn default() -> Self {
        Self {
            decoder_ctx: Default::default(),
            index: Default::default(),
        }
    }
}

struct MediaStreams {
    audio_stream: StreamInfo,
    video_stream: StreamInfo,
    subtitle_stream: StreamInfo,
    attachment_stream: StreamInfo,
    nb_stream: StreamInfo,
    data_stream: StreamInfo,
    unknown_streams: Vec<StreamInfo>,
}

trait MediaBuffer {
    type Item;
    fn pop(&self) -> Option<Self::Item>;
    fn push(&self, item: Self::Item) -> Result<(), Self::Item>;
    fn is_empty(&self) -> bool;
    fn is_full(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioFrame {
    pub format: i32,
    /// Audio content, usually FLTP format, type: `AVSampleFormat`
    pub data: Vec<f32>,
    /// display timestamp
    pub pts: i64,
    /// Pts in milliseconds
    pub pts_millis: i64,
    pub sample_rate: i32,
    pub channels: u8,
    pub channel_layout: u8,
}

pub struct AudioBuffer {
    inner: ArrayQueue<AudioFrame>,
}

impl AudioBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            inner: ArrayQueue::<AudioFrame>::new(size),
        }
    }
}

impl Deref for AudioBuffer {
    type Target = ArrayQueue<AudioFrame>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AudioBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct VideoFrame {
    pub format: i32,
    pub data: [Vec<u8>; 8],
    pub width: usize,
    pub height: usize,
    pub pts: i64,
    /// Pts in milliseconds
    pub pts_millis: i64,
}

pub struct VideoBuffer {
    inner: ArrayQueue<VideoFrame>,
}

impl VideoBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            inner: ArrayQueue::<VideoFrame>::new(size),
        }
    }
}

impl Deref for VideoBuffer {
    type Target = ArrayQueue<VideoFrame>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for VideoBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct SubtitleFrame {
    pub format: i32,
    pub data: [Vec<u8>; 8],
    pub pts: i64,
    /// Pts in milliseconds
    pub pts_millis: i64,
}

pub struct SubtitleBuffer {
    inner: ArrayQueue<SubtitleFrame>,
}

impl SubtitleBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            inner: ArrayQueue::<SubtitleFrame>::new(size),
        }
    }
}

impl Deref for SubtitleBuffer {
    type Target = ArrayQueue<SubtitleFrame>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SubtitleBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
