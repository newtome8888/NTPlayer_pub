use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use sdl2::{
    image::LoadSurface,
    mouse::MouseButton,
    rect::Rect,
    render::Canvas,
    surface::Surface,
    video::{Window, WindowPos},
};

use crate::{
    entity::EventMessage,
    global::EVENT_CHANNEL,
    ui::{Button, MouseUpParam, TControl},
    util::error::{safe_send, SuperError},
};

pub struct PlayButton {
    inner: Button,
}

impl PlayButton {
    pub fn default(canvas: Rc<RefCell<Canvas<Window>>>) -> Result<Self, SuperError> {
        Self::new(0, 0, 50, 30, canvas)
    }

    pub fn new(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        canvas: Rc<RefCell<Canvas<Window>>>,
    ) -> Result<Self, SuperError> {
        let inner = Button::new(x, y, width, height, canvas)?;

        Ok(Self { inner })
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;

        self
    }

    /// This method should be put at the end of method chain,
    /// otherwise the position will be affected by new size of window
    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.set_position(WindowPos::Positioned(x), WindowPos::Positioned(y));

        self
    }

    pub fn on_mouse_up(&mut self, params: &MouseUpParam) -> Result<bool, SuperError> {
        if !self.inner.on_mouse_up(params)? {
            return Ok(false);
        }

        match params.mouse_btn {
            MouseButton::Left => {
                // Play button is clicked, open file
                let path = rfd::FileDialog::new().pick_file();
                if let Some(path) = path {
                    safe_send(EVENT_CHANNEL.0.send(EventMessage::FileOpened(path)));
                }
            }
            _ => {}
        }

        Ok(true)
    }

    pub fn render(&mut self) -> Result<(), SuperError> {
        let mut canvas_mut = self.canvas.borrow_mut();
        let texture_creator = canvas_mut.texture_creator();
        let sfs = Surface::from_file("./assets/play_black_circle.png")?;
        let texture = texture_creator.create_texture_from_surface(sfs)?;

        canvas_mut.clear();
        canvas_mut.copy(
            &texture,
            None,
            Rect::new(self.x, self.y, self.width, self.height),
        )?;

        Ok(())
    }
}

impl Deref for PlayButton {
    type Target = Button;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PlayButton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
