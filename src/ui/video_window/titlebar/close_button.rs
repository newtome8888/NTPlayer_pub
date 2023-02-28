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
    video::Window,
};

use crate::{
    ui::{Button, MouseMotionParam, TButton, TControl, MouseUpParam},
    util::error::{SuperError, safe_send}, global::EVENT_CHANNEL, entity::EventMessage,
};

pub struct CloseButton {
    inner: Button,
    selected: bool,
}

impl CloseButton {
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
            canvas.set_draw_color(Color::RGB(220, 20, 60));

            canvas.fill_rect(button_rect)?;
        }

        // draw X shape, the size of X shape is const value 20X20
        let (center_x, center_y) = self.center;
        let step = 5;
        canvas.set_draw_color(Color::WHITE);
        canvas.draw_line(
            Point::new(center_x - step, center_y - step),
            Point::new(center_x + step, center_y + step),
        )?;
        canvas.draw_line(
            Point::new(center_x - step, center_y + step),
            Point::new(center_x + step, center_y - step),
        )?;

        Ok(true)
    }

    pub fn on_mouse_motion(&mut self, params: &MouseMotionParam) -> Result<bool, SuperError> {
        self.selected = self.inner.on_mouse_motion(params)?;

        Ok(true)
    }

    pub fn on_mouse_up(&mut self, params: &MouseUpParam) -> Result<bool, SuperError>{
        if ! self.inner.on_mouse_up(params)?{
            return Ok(false);
        }

        safe_send(EVENT_CHANNEL.0.send(EventMessage::ExitVideoWindow));

        Ok(true)
    }
}

impl TButton for CloseButton {}

impl Deref for CloseButton {
    type Target = Button;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CloseButton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
