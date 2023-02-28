use sdl2::{
    event::{Event, WindowEvent},
    keyboard::{Keycode, Mod},
    EventPump, Sdl, video::FullscreenType,
};

use super::MainLoopState;
use crate::{
    entity::EventMessage,
    global::{EVENT_CHANNEL, INIT_HEIGHT, INIT_WIDTH},
    ui::{
        start_window::StartWindow, video_window::VideoWindow, MouseDownParam, MouseMotionParam,
        MouseUpParam, MouseWheelParam,
    },
    util::error::{safe_send, SuperError},
};

pub(in crate::app) struct SdlEvents {
    event_pump: EventPump,
}

impl SdlEvents {
    pub(in crate::app) fn new(ctx: &Sdl) -> Result<Self, SuperError> {
        let event_pump = ctx.event_pump()?;

        Ok(Self { event_pump })
    }

    /// Handler for sdl events, if the return value is Ok(false),
    /// means the main loop should be terminated, otherwise just continue
    pub(in crate::app) fn handle_events(
        &mut self,
        start_window: &mut StartWindow,
        video_window: &mut Option<VideoWindow>,
    ) -> Result<MainLoopState, SuperError> {
        let sender = &EVENT_CHANNEL.0;

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return Ok(MainLoopState::Quit),
                Event::KeyDown {
                    keycode,
                    window_id,
                    keymod,
                    ..
                } => {
                    println!("mod: {}", keymod);
                    match keycode {
                        Some(Keycode::Escape) => {
                            if window_id == start_window.id {
                                return Ok(MainLoopState::Quit);
                            }
                            if let Some(wind) = video_window {
                                if window_id == wind.id {
                                    wind.set_fullscreen(FullscreenType::Off);
                                }
                            }
                        }
                        Some(Keycode::Up) => {
                            safe_send(sender.send(EventMessage::UpVolume));
                        }
                        Some(Keycode::Down) => {
                            safe_send(sender.send(EventMessage::DownVolume));
                        }
                        Some(Keycode::Left) => {
                            safe_send(sender.send(EventMessage::Rewind));
                        }
                        Some(Keycode::Right) => {
                            safe_send(sender.send(EventMessage::Forward));
                        }
                        Some(Keycode::F4) => {
                            return Ok(MainLoopState::Quit);
                        }
                        _ => {}
                    }
                }
                Event::MouseMotion {
                    timestamp,
                    window_id,
                    which,
                    mousestate,
                    x,
                    y,
                    xrel,
                    yrel,
                } => {
                    let params = MouseMotionParam {
                        timestamp,
                        window_id,
                        which,
                        mousestate,
                        x,
                        y,
                        xrel,
                        yrel,
                    };

                    start_window.on_mouse_motion(&params)?;
                    if let Some(window) = video_window {
                        window.on_mouse_motion(&params)?;
                    }
                }
                Event::MouseButtonUp {
                    timestamp,
                    window_id,
                    which,
                    mouse_btn,
                    clicks,
                    x,
                    y,
                } => {
                    let params = MouseUpParam {
                        timestamp,
                        window_id,
                        which,
                        mouse_btn,
                        clicks,
                        x,
                        y,
                    };
                    start_window.on_mouse_up(&params)?;
                    if let Some(window) = video_window {
                        window.on_mouse_up(&params)?;
                    }
                }
                Event::MouseButtonDown {
                    timestamp,
                    window_id,
                    which,
                    mouse_btn,
                    clicks,
                    x,
                    y,
                } => {
                    let params = MouseDownParam {
                        timestamp,
                        window_id,
                        which,
                        mouse_btn,
                        clicks,
                        x,
                        y,
                    };

                    start_window.on_mouse_down(&params)?;
                    if let Some(window) = video_window {
                        window.on_mouse_down(&params)?;
                    }
                }
                Event::MouseWheel {
                    timestamp,
                    window_id,
                    which,
                    x,
                    y,
                    direction,
                } => {
                    let params = MouseWheelParam {
                        timestamp,
                        window_id,
                        which,
                        x,
                        y,
                        direction,
                    };

                    start_window.on_mouse_wheel(&params)?;
                    if let Some(window) = video_window {
                        window.on_mouse_wheel(&params)?;
                    }
                }
                Event::Window {
                    timestamp: _,
                    window_id,
                    win_event,
                } => match win_event {
                    WindowEvent::Resized(width, height) => {
                        if let Some(window) = video_window {
                            if window_id == window.id {
                                window.on_resized(width as u32, height as u32);
                            }
                        }
                    }
                    _ => {}
                },
                _ => return Ok(MainLoopState::Continue),
            }
        }
        Ok(MainLoopState::Continue)
    }
}
