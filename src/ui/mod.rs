/// # Warning
/// Don't remove the symbol `{}` which around the canvas borrow_mut codes,
/// they are used to make sure the mutable borrow value will be released at
/// once to avaoid multiple mut borrow at the same time.
pub mod components;
pub mod foundation;
pub mod start_window;
pub mod video_window;

use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use sdl2::{
    mouse::{MouseButton, MouseState, MouseWheelDirection},
    render::Canvas,
    video::{Window, WindowPos},
};

use crate::util::error::SuperError;

pub trait TControl {
    /// Handle mouse botton down event
    /// # Returns
    /// * `bool` - true if the event is handled, false if the event is rejected.
    /// The reason for rejection may be the cursor is not in the area of current control,
    /// or other reasons.
    /// * `SuperError` - the error information
    fn on_mouse_down(&mut self, params: &MouseDownParam) -> Result<bool, SuperError>;
    /// Handle mouse botton up event
    /// # Returns
    /// * `bool` - true if the event is handled, false if the event is rejected.
    /// The reason for rejection may be the cursor is not in the area of current control,
    /// or other reasons.
    /// * `SuperError` - the error information
    fn on_mouse_up(&mut self, params: &MouseUpParam) -> Result<bool, SuperError>;
    /// Handle mouse move event
    /// # Returns
    /// * `bool` - true if the event is handled, false if the event is rejected.
    /// The reason for rejection may be the cursor is not in the area of current control,
    /// or other reasons.
    /// * `SuperError` - the error information
    fn on_mouse_motion(&mut self, params: &MouseMotionParam) -> Result<bool, SuperError>;
    /// Handle mouse wheel event
    /// # Returns
    /// * `bool` - true if the event is handled, false if the event is rejected.
    /// The reason for rejection may be the cursor is not in the area of current control,
    /// or other reasons.
    /// * `SuperError` - the error information
    fn on_mouse_wheel(&mut self, params: &MouseWheelParam) -> Result<bool, SuperError>;
    /// Handle control resized event
    /// # Returns
    /// * `bool` - true if the event is handled, false if the event is rejected.
    /// The reason for rejection may be the cursor is not in the area of current control,
    /// or other reasons.
    /// * `SuperError` - the error information
    fn on_resized(&mut self, width: u32, height: u32) -> Result<bool, SuperError>;
    /// Set the size of current control
    /// # Arguments
    /// * `width` - the new width of the control
    /// * `height` - the new height of the control
    fn set_size(&mut self, width: u32, height: u32);
    /// Set the position of current control
    /// # Arguments
    /// * `x` - the new x position of the control
    /// * `y` - the new y position of the control
    fn set_position(&mut self, x: WindowPos, y: WindowPos);
    /// Render the data of current control to the screen
    /// # Returns
    /// * `bool` - true if the event is handled, false if the event is rejected.
    /// The reason for rejection may be the cursor is not in the area of current control,
    /// or other reasons.
    /// * `SuperError` - the error information
    /// # Warning
    /// * This function is not implemented yet,
    /// any one want to call this function should overwrite it with your own implementation
    fn render(&mut self) -> Result<bool, SuperError>;
    /// Get the position of current control
    /// # Returns
    /// * `i32` - the x position of control
    /// * `i32` - the y position of control
    fn get_position(&self) -> (i32, i32);
    /// Get the size of current control
    /// # Returns
    /// * `u32` - the width of control
    /// * `u32` - the height of control
    fn get_size(&self) -> (u32, u32);
    /// Get the center position of current control
    /// # Returns
    /// * `i32` - the x position of the center of the control
    /// * `i32` - the y position of the center of the control
    fn get_center(&self) -> (i32, i32);
}

pub trait TRectangleControl: TControl {
    /// Check if a point is in the area of specified rectangle.
    /// # Arguments:
    /// * `x`: The x coordinate of the point
    /// * `y`: The y coordinate of the point
    /// # Returns
    /// * `true` if the point is in the area of current control
    /// * `false` if the point is not in the area of current control
    fn is_in_area(&self, x: i32, y: i32) -> bool;

    /// Compute the distance between specified point and specified rectangle
    /// # Arguments:
    /// * `x`: The x coordinate of the cursor
    /// * `y`: The y coordinate of the cursor
    /// # Returns
    /// * `u32` - the distance between the point and the rectangle
    /// * `DistanceDirection` - the direction of the point located at the rectangle
    fn get_distance(&self, x: i32, y: i32) -> (u32, DistanceDirection);
}

pub trait TButton {}

pub trait TCircularControl: TControl {
    /// Check if a point is in the specified circular
    /// # Arguments
    /// * `x`: The x coordinate of the point
    /// * `y`: The y coordinate of the point
    /// * `center_x`: The x coordinate of the circular center
    /// * `center_y`: The y coordinate of the circular center
    /// * `radius`: The radius of the circular
    fn is_in_area(&self, x: i32, y: i32) -> bool;

    /// Get the distance between a point and circular
    /// # Arguments
    /// * `x`: The x coordinate of the point
    /// * `y`: The y coordinate of the point
    /// * `center_x`: The x coordinate of the circular center
    /// * `center_y`: The y coordinate of the circular center
    /// * `radius`: The radius of the circular
    fn get_distance(&self, x: i32, y: i32) -> f32;
}

pub struct RectangleControl {
    canvas: Rc<RefCell<Canvas<Window>>>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    center: (i32, i32),
}

impl RectangleControl {
    pub fn new(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        canvas: Rc<RefCell<Canvas<Window>>>,
    ) -> Result<Self, SuperError> {
        let center_x = x + width as i32 / 2;
        let center_y = y + height as i32 / 2;

        Ok(Self {
            x,
            y,
            width,
            height,
            canvas,
            center: (center_x, center_y),
        })
    }
}

impl TControl for RectangleControl {
    fn on_mouse_down(&mut self, params: &MouseDownParam) -> Result<bool, SuperError> {
        if !self.is_in_area(params.x, params.y) {
            return Ok(false);
        }

        Ok(true)
    }

    fn on_mouse_up(&mut self, params: &MouseUpParam) -> Result<bool, SuperError> {
        if !self.is_in_area(params.x, params.y) {
            return Ok(false);
        }

        Ok(true)
    }

    fn on_mouse_motion(&mut self, params: &MouseMotionParam) -> Result<bool, SuperError> {
        if !self.is_in_area(params.x, params.y) {
            return Ok(false);
        }

        Ok(true)
    }

    fn on_mouse_wheel(&mut self, params: &MouseWheelParam) -> Result<bool, SuperError> {
        if !self.is_in_area(params.x, params.y) {
            return Ok(false);
        }

        Ok(true)
    }

    fn on_resized(&mut self, _width: u32, _height: u32) -> Result<bool, SuperError> {
        todo!()
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        self.center = (
            self.x + self.width as i32 / 2,
            self.y + self.height as i32 / 2,
        );
    }

    fn set_position(&mut self, x: WindowPos, y: WindowPos) {
        if let WindowPos::Positioned(x) = x {
            self.x = x;
        }

        if let WindowPos::Positioned(y) = y {
            self.y = y;
        }

        self.center = (
            self.x + self.width as i32 / 2,
            self.y + self.height as i32 / 2,
        );
    }

    fn render(&mut self) -> Result<bool, SuperError> {
        todo!()
    }

    fn get_position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_center(&self) -> (i32, i32) {
        self.center
    }
}

impl TRectangleControl for RectangleControl {
    fn is_in_area(&self, x: i32, y: i32) -> bool {
        (x >= self.x && x <= self.x + self.width as i32)
            && (y >= self.y && y <= self.y + self.height as i32)
    }

    fn get_distance(&self, x: i32, y: i32) -> (u32, DistanceDirection) {
        let x_start = self.x;
        let x_end = self.x + self.width as i32;
        let y_start = self.y;
        let y_end = self.y + self.height as i32;

        let (distance, direction) = if x < x_start {
            ((x - x_start) as u32, DistanceDirection::Left)
        } else if x > x_end {
            ((x_end - x) as u32, DistanceDirection::Right)
        } else if y < y_start {
            ((y_start - y) as u32, DistanceDirection::Up)
        } else if y > y_end {
            ((y - y_end) as u32, DistanceDirection::Down)
        } else {
            (0, DistanceDirection::Inside)
        };

        (distance, direction)
    }
}

pub struct Button {
    inner: RectangleControl,
}

impl Button {
    pub fn new(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        canvas: Rc<RefCell<Canvas<Window>>>,
    ) -> Result<Self, SuperError> {
        let inner = RectangleControl::new(x, y, width, height, canvas)?;

        Ok(Self { inner })
    }
}

impl TButton for Button {}

impl Deref for Button {
    type Target = RectangleControl;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Button {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct CircularControl {
    center_x: i32,
    center_y: i32,
    radius: u32,
}
impl CircularControl {}

impl TCircularControl for CircularControl {
    /// Check if a point is in the specified circular
    /// # Arguments
    /// * `x`: The x coordinate of the point
    /// * `y`: The y coordinate of the point
    fn is_in_area(&self, x: i32, y: i32) -> bool {
        let x = x as i64;
        let y = y as i64;
        let center_x = self.center_x as i64;
        let center_y = self.center_y as i64;
        let radius = self.radius as i64;

        let x_distance = x - center_x;
        let y_distance = y - center_y;

        x_distance * x_distance + y_distance * y_distance <= radius * radius
    }

    /// Get the distance between a point and circular
    /// # Arguments
    /// * `x`: The x coordinate of the point
    /// * `y`: The y coordinate of the point
    fn get_distance(&self, x: i32, y: i32) -> f32 {
        let x = x as i64;
        let y = y as i64;
        let center_x = self.center_x as i64;
        let center_y = self.center_y as i64;
        let radius = self.radius as f64;

        let x_distance = x - center_x;
        let y_distance = y - center_y;

        let sum_square = (x_distance * x_distance + y_distance * y_distance) as f64;
        let distance = (sum_square.sqrt() - radius) as f32;

        distance
    }
}

impl TControl for CircularControl {
    fn on_mouse_down(&mut self, _params: &MouseDownParam) -> Result<bool, SuperError> {
        todo!()
    }

    fn on_mouse_up(&mut self, _params: &MouseUpParam) -> Result<bool, SuperError> {
        todo!()
    }

    fn on_mouse_motion(&mut self, _params: &MouseMotionParam) -> Result<bool, SuperError> {
        todo!()
    }

    fn on_mouse_wheel(&mut self, _params: &MouseWheelParam) -> Result<bool, SuperError> {
        todo!()
    }

    fn set_size(&mut self, _width: u32, _height: u32) {
        todo!()
    }

    fn set_position(&mut self, _x: WindowPos, _y: WindowPos) {
        todo!()
    }

    fn render(&mut self) -> Result<bool, SuperError> {
        todo!()
    }

    fn get_position(&self) -> (i32, i32) {
        todo!()
    }

    fn get_size(&self) -> (u32, u32) {
        todo!()
    }

    fn get_center(&self) -> (i32, i32) {
        todo!()
    }

    fn on_resized(&mut self, width: u32, height: u32) -> Result<bool, SuperError> {
        todo!()
    }
}

pub struct MouseDownParam {
    pub timestamp: u32,
    pub window_id: u32,
    pub which: u32,
    pub mouse_btn: MouseButton,
    pub clicks: u8,
    pub x: i32,
    pub y: i32,
}

pub struct MouseUpParam {
    pub timestamp: u32,
    pub window_id: u32,
    pub which: u32,
    pub mouse_btn: MouseButton,
    pub clicks: u8,
    pub x: i32,
    pub y: i32,
}

pub struct MouseMotionParam {
    pub timestamp: u32,
    pub window_id: u32,
    pub which: u32,
    pub mousestate: MouseState,
    pub x: i32,
    pub y: i32,
    pub xrel: i32,
    pub yrel: i32,
}

pub struct MouseWheelParam {
    pub timestamp: u32,
    pub window_id: u32,
    pub which: u32,
    pub x: i32,
    pub y: i32,
    pub direction: MouseWheelDirection,
}

pub enum DistanceDirection {
    Inside,
    Up,
    Down,
    Left,
    Right,
}
