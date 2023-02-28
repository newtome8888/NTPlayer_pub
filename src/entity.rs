use sdl2::video::WindowPos;
use std::path::PathBuf;

use crate::media::decoder::{AudioFrame, SubtitleFrame, VideoFrame};

/// Message types for application related events
pub enum EventMessage {
    /// Exit application
    Quit,
    /// Exit video window and return back to the start window
    ExitVideoWindow,

    /// Show error dialog
    ShowError(String),

    // For media state control
    Play(PathBuf),
    Pause,
    Resume,
    Stop,
    Forward,
    Rewind,

    // Indicate that forward or rewind operation has been completed
    SeekFinished,

    // File
    FileOpened(PathBuf),
    DirOpened(Vec<PathBuf>),

    // Rendering
    RenderVideo(VideoFrame),
    RenderAudio(AudioFrame),
    RenderSubtitle(SubtitleFrame),

    // UI layout
    Resize((u32, u32)),
    SetPosition {
        x: WindowPos,
        y: WindowPos,
    },

    // Volume control
    UpVolume,
    DownVolume,
}

pub struct MediaSelectedData {
    pub path: PathBuf,
}

pub struct MediaItemDoubleClickedData {
    pub path: PathBuf,
}
