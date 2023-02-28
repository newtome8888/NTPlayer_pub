use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use log::debug;
use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::{FullscreenType, Window},
};

use crate::{
    entity::EventMessage,
    global::EVENT_CHANNEL,
    ui::{Button, MouseMotionParam, MouseUpParam, TButton, TControl},
    util::error::{safe_send, SuperError},
};

pub struct MaximizeButton {
    inner: Button,
    selected: bool,
}

impl MaximizeButton {
    pub fn new(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        canvas: Rc<RefCell<Canvas<Window>>>,
    ) -> Result<Self, SuperError> {
        let inner = Button::new(x, y, width, height, canvas)?;

        Ok(Self {
            inner,
            selected: false,
        })
    }

    pub fn render(&mut self) -> Result<bool, SuperError> {
        let mut canvas = self.canvas.borrow_mut();
        let button_rect = Rect::new(self.x, self.y, self.width, self.height);

        // draw background
        if self.selected {
            canvas.set_draw_color(Color::RGB(51, 51, 255));

            canvas.fill_rect(button_rect)?;
        }

        // draw rect shape
        canvas.set_draw_color(Color::WHITE);

        let (center_x, center_y) = self.center;
        let step = 5;
        let wind = canvas.window_mut();
        match wind.fullscreen_state() {
            FullscreenType::Off => {}
            _ => {
                let points = [
                    Point::new(center_x - step + 2, center_y - step - 2),
                    Point::new(center_x + step + 2, center_y - step - 2),
                    Point::new(center_x + step + 2, center_y + step - 2),
                ];
                canvas.draw_lines(&points[..])?;
            }
        }

        let inner_rect = Rect::new(
            center_x - step,
            center_y - step,
            step as u32 * 2,
            step as u32 * 2,
        );

        canvas.draw_rect(inner_rect)?;

        Ok(true)
    }

    pub fn on_mouse_motion(&mut self, params: &MouseMotionParam) -> Result<bool, SuperError> {
        self.selected = self.inner.on_mouse_motion(params)?;

        Ok(true)
    }

    pub fn on_mouse_up(&mut self, params: &MouseUpParam) -> Result<bool, SuperError> {
        if !self.inner.on_mouse_up(params)? {
            return Ok(false);
        }

        let mut canvas = self.canvas.borrow_mut();
        let wind = canvas.window_mut();
        match wind.fullscreen_state() {
            FullscreenType::Off => {
                wind.set_fullscreen(FullscreenType::Desktop)?;
            }
            _ => {
                wind.set_fullscreen(FullscreenType::Off)?;
            }
        }

        Ok(true)
    }
}

impl TButton for MaximizeButton {}

impl Deref for MaximizeButton {
    type Target = Button;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MaximizeButton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
