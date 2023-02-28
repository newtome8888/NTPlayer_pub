mod playbar;
mod playbox;
mod progressbar;
mod titlebar;

use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use log::error;
use sdl2::{
    image::LoadSurface,
    pixels::Color,
    render::Canvas,
    surface::Surface,
    video::{FullscreenType, Window, WindowPos},
    VideoSubsystem,
};

use crate::media::decoder::VideoFrame;
use crate::util::error::SuperError;
use crate::{
    global::{APP_NAME, INIT_HEIGHT, INIT_WIDTH, LOGO_PATH},
    util::error::handle_result,
};

use self::playbar::PlayBar;
use self::playbox::PlayBox;
use self::progressbar::ProgressBar;
use self::titlebar::TitleBar;

use super::{MouseDownParam, MouseMotionParam, MouseUpParam, MouseWheelParam, RectangleControl};

pub const BACKGROUND_COLOR: Color = Color::RGB(0, 0, 0);

pub struct VideoWindow {
    pub id: u32,
    inner: RectangleControl,
    titlebar: TitleBar,
    playbar: PlayBar,
    progressbar: ProgressBar,
    playbox: PlayBox,
}

impl VideoWindow {
    pub fn new(sys: &VideoSubsystem) -> Result<Self, SuperError> {
        let wind = Self::prepare_window(sys)?;
        let window_id = wind.id();
        let (x, y) = wind.position();
        let (width, height) = wind.size();

        let canvas = Self::prepare_canvas(wind)?;
        let canvas = Rc::new(RefCell::new(canvas));
        let play_box = PlayBox::new(0, 0, INIT_WIDTH, INIT_HEIGHT, canvas.clone())?;

        Ok(Self {
            titlebar: TitleBar::new(canvas.clone(), None, None, None, None)?,
            playbar: PlayBar,
            progressbar: ProgressBar,
            playbox: play_box,
            id: window_id,
            inner: RectangleControl::new(x, y, width, height, canvas.clone())?,
        })
    }

    pub fn show(&mut self) {
        self.canvas.borrow_mut().window_mut().show();
    }

    pub fn hide(&mut self) {
        self.canvas.borrow_mut().window_mut().hide();
    }

    pub fn set_logo(&mut self, path: &str) -> Result<(), SuperError> {
        let logo = Surface::from_file(path)?;
        self.canvas.borrow_mut().window_mut().set_icon(logo);

        Ok(())
    }

    pub fn update_video_frame(&mut self, frame: VideoFrame) {
        self.playbox.update_frame(frame);
    }

    fn prepare_window(sys: &VideoSubsystem) -> Result<Window, SuperError> {
        let mut wind = sys
            .window("NT Player", INIT_WIDTH, INIT_HEIGHT)
            .borderless()
            .allow_highdpi()
            .position_centered()
            .resizable()
            .opengl()
            .build()?;

        wind.set_title(APP_NAME)?;

        let logo = Surface::from_file(LOGO_PATH)?;
        wind.set_icon(logo);

        Ok(wind)
    }

    fn prepare_canvas(wind: Window) -> Result<Canvas<Window>, SuperError> {
        let mut canvas = wind.into_canvas().build()?;
        canvas.set_draw_color(BACKGROUND_COLOR);

        Ok(canvas)
    }

    pub fn on_mouse_down(&mut self, params: &MouseDownParam) -> Result<bool, SuperError> {
        if params.window_id != self.id {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn on_mouse_up(&mut self, params: &MouseUpParam) -> Result<bool, SuperError> {
        if params.window_id != self.id {
            return Ok(false);
        }

        self.titlebar.on_mouse_up(params)?;

        Ok(true)
    }

    pub fn on_mouse_motion(&mut self, params: &MouseMotionParam) -> Result<bool, SuperError> {
        if params.window_id != self.id {
            return Ok(false);
        }

        self.titlebar.on_mouse_motion(params)?;
        Ok(true)
    }

    pub fn on_mouse_wheel(&mut self, params: &MouseWheelParam) -> Result<bool, SuperError> {
        if params.window_id != self.id {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        let result = {
            self.canvas
                .borrow_mut()
                .window_mut()
                .set_size(width, height)
        };
        match result {
            Ok(_) => {
                self.on_resized(width, height);
            }
            Err(err) => error!("set window size failed: {:?}", err),
        }
    }

    pub fn on_resized(&mut self, width: u32, height: u32) {
        // Adjust playbox size
        self.playbox.set_size(width, height);

        // Adjuist titlebar size
        let tb_height = self.titlebar.height;
        self.titlebar.set_size(width, tb_height);
    }

    pub fn set_position(&mut self, x: WindowPos, y: WindowPos) {
        self.canvas.borrow_mut().window_mut().set_position(x, y);
    }

    pub fn render(&mut self) -> Result<bool, SuperError> {
        self.canvas.borrow_mut().set_draw_color(Color::BLACK);
        self.canvas.borrow_mut().clear();

        // Render content
        self.playbox.render()?;
        self.titlebar.render()?;

        // Display on screen
        self.canvas.borrow_mut().present();

        Ok(true)
    }

    pub fn set_fullscreen(&mut self, fs_type: FullscreenType) {
        if let Err(err) = self
            .canvas
            .borrow_mut()
            .window_mut()
            .set_fullscreen(fs_type)
        {
            error!("Failed to set fullscreen state, error: {:?}", err);
        }
    }
}

impl Deref for VideoWindow {
    type Target = RectangleControl;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for VideoWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
