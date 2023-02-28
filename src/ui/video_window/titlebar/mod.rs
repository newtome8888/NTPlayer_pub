mod close_button;
mod maximize_button;
mod minimize_button;

use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use sdl2::{
    render::Canvas,
    video::{Window, WindowPos},
};

use self::{
    close_button::CloseButton, maximize_button::MaximizeButton, minimize_button::MinimizeButton,
};
use crate::{
    ui::{MouseDownParam, MouseMotionParam, MouseUpParam, RectangleControl, TControl},
    util::error::SuperError,
};

pub struct TitleBar {
    inner: RectangleControl,
    close_button: CloseButton,
    maxmize_button: MaximizeButton,
    minimize_button: MinimizeButton,
    /// Indicate if user is operating on this control
    op_flag: Arc<AtomicBool>,
}

impl TitleBar {
    pub fn new<I, U>(
        canvas: Rc<RefCell<Canvas<Window>>>,
        x: I,
        y: I,
        width: U,
        height: U,
    ) -> Result<Self, SuperError>
    where
        I: Into<Option<i32>>,
        U: Into<Option<u32>>,
    {
        let (window_width, _) = canvas.clone().borrow().output_size()?;
        let x = x.into().unwrap_or(0);
        let y = y.into().unwrap_or(0);
        let width = width.into().unwrap_or(window_width);
        let height = height.into().unwrap_or(40);

        let inner = RectangleControl::new(x, y, width, height, canvas.clone())?;

        let btn_width = 40;
        let btn_height = 30;
        let btn_y = 0;

        let close_btn_x = (window_width - btn_width) as i32;
        let max_btn_x = close_btn_x - btn_width as i32 - 5;
        let mini_button_x = max_btn_x - btn_width as i32 - 5;

        let close_button =
            CloseButton::new(close_btn_x, btn_y, btn_width, btn_height, canvas.clone())?;
        let maximize_button =
            MaximizeButton::new(max_btn_x, btn_y, btn_width, btn_height, canvas.clone())?;
        let minimize_button =
            MinimizeButton::new(mini_button_x, btn_y, btn_width, btn_height, canvas.clone())?;

        Ok(Self {
            inner,
            close_button,
            maxmize_button: maximize_button,
            minimize_button,
            op_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.inner.set_size(width, height);

        // Adjust position of close button since the titlebar size has been changed
        let (clbtn_width, _) = self.close_button.get_size();
        let clbtn_x = self.x + width as i32 - clbtn_width as i32;
        self.close_button
            .set_position(WindowPos::Positioned(clbtn_x), WindowPos::Positioned(0));

        // Adjust position of maxmize button since the titlebar size has been changed
        let (mxbtn_width, _) = self.maxmize_button.get_size();
        let mxbtn_x = clbtn_x - mxbtn_width as i32;
        self.maxmize_button
            .set_position(WindowPos::Positioned(mxbtn_x), WindowPos::Positioned(0));

        // Adjust position of minimize button since the titlebar size has been changed
        let (mibtn_width, _) = self.minimize_button.get_size();
        let mibtn_x = mxbtn_x - mibtn_width as i32;
        self.minimize_button
            .set_position(WindowPos::Positioned(mibtn_x), WindowPos::Positioned(0));
    }

    pub fn render(&mut self) -> Result<bool, SuperError> {
        // If user is currently operating on canvas, show sub components
        if self.op_flag.load(Ordering::Acquire) {
            self.close_button.render()?;
            self.maxmize_button.render()?;
            self.minimize_button.render()?;
        }

        Ok(true)
    }

    pub fn on_mouse_motion(&mut self, params: &MouseMotionParam) -> Result<bool, SuperError> {
        if !self.inner.on_mouse_motion(params)? {
            self.op_flag.store(false, Ordering::Release);
            return Ok(false);
        }

        self.op_flag.store(true, Ordering::Release);

        self.close_button.on_mouse_motion(params)?;
        self.maxmize_button.on_mouse_motion(params)?;
        self.minimize_button.on_mouse_motion(params)?;

        Ok(true)
    }

    pub fn on_mouse_up(&mut self, params: &MouseUpParam) -> Result<bool, SuperError> {
        if !self.inner.on_mouse_up(params)? {
            self.op_flag.store(false, Ordering::Release);
            return Ok(false);
        }
        
        self.op_flag.store(true, Ordering::Release);

        self.close_button.on_mouse_up(params)?;
        self.maxmize_button.on_mouse_up(params)?;
        self.minimize_button.on_mouse_up(params)?;

        Ok(true)
    }

    pub fn on_mouse_down(&mut self, params: &MouseDownParam) -> Result<bool, SuperError> {
        if !self.inner.on_mouse_down(params)? {
            self.op_flag.store(false, Ordering::Release);
            return Ok(false);
        }

        self.op_flag.store(true, Ordering::Release);

        Ok(true)
    }
}

impl Deref for TitleBar {
    type Target = RectangleControl;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TitleBar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
