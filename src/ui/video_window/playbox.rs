use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use log::warn;
use rsmpeg::ffi::AVPixelFormat_AV_PIX_FMT_YUV420P as AVPIXELFORMAT_AV_PIX_FMT_YUV420P;
use sdl2::{
    pixels::PixelFormatEnum,
    rect::Rect,
    render::Canvas,
    video::{Window, WindowPos},
};

use crate::{
    entity::EventMessage,
    global::EVENT_CHANNEL,
    media::decoder::VideoFrame,
    ui::{RectangleControl, TControl},
    util::error::{safe_send, SuperError},
};

pub struct PlayBox {
    inner: RectangleControl,
    frame: Option<VideoFrame>,
}

impl PlayBox {
    pub fn new(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        canvas: Rc<RefCell<Canvas<Window>>>,
    ) -> Result<Self, SuperError> {
        Ok(Self {
            inner: RectangleControl::new(x, y, width, height, canvas.clone())?,
            frame: None,
        })
    }

    pub fn update_frame(&mut self, frame: VideoFrame) {
        self.frame = Some(frame);
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.inner.set_size(width, height);

        let result = EVENT_CHANNEL.0.send(EventMessage::SetPosition {
            x: WindowPos::Centered,
            y: WindowPos::Centered,
        });

        safe_send(result);
    }

    pub fn render(&mut self) -> Result<bool, SuperError> {
        if self.frame.is_none() {
            return Ok(false);
        }

        let mut canvas_mut = self.canvas.borrow_mut();
        let texture_creator = canvas_mut.texture_creator();
        let wind = canvas_mut.window_mut();

        let frame = self.frame.as_ref().unwrap();
        let frame_width = frame.width as u32;
        let frame_height = frame.height as u32;

        let mut texture = texture_creator.create_texture_streaming(
            PixelFormatEnum::IYUV,
            frame_width,
            frame_height,
        )?;
        match frame.format {
            AVPIXELFORMAT_AV_PIX_FMT_YUV420P => {
                let data = &frame.data;
                let ypitch = frame.width;
                let upitch = ypitch / 2;
                let vpitch = ypitch / 2;

                texture.update_yuv(None, &data[0], ypitch, &data[1], upitch, &data[2], vpitch)?;
            }
            _ => {
                warn!("unknown pixel format: {}", frame.format);
                return Ok(false);
            }
        }

        let (width, height) = self.compute_render_size(frame_width, frame_height)?;
        let (x, y) = self.compute_render_position(width, height);
        canvas_mut.copy(&texture, None, Rect::new(x, y, width, height))?;
        Ok(true)
    }

    fn compute_render_size(
        &self,
        frame_width: u32,
        frame_height: u32,
    ) -> Result<(u32, u32), SuperError> {
        let ratio = frame_width / frame_height;
        let height = self.width / ratio;

        Ok((self.width, height))
    }

    fn compute_render_position(&self, width: u32, height: u32) -> (i32, i32) {
        let (center_x, center_y) = self.center;
        (center_x - width as i32 / 2, center_y - height as i32 / 2)
    }
}

impl Deref for PlayBox {
    type Target = RectangleControl;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PlayBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
