use crate::inputs::{Key, MouseButton, MouseWheelDirection};

pub trait EventSystem: std::fmt::Debug {
    fn invoke_mouse_button_event(&self, e: MouseButtonEvent);

    fn invoke_mouse_wheel_event(&self, e: MouseWheelEvent);

    fn invoke_mouse_move_event(&self, e: MouseMoveEvent);

    fn invoke_key_event(&self, e: KeyEvent);

    fn invoke_window_resize_event(&self, e: WindowResizeEvent);

    fn invoke_window_event(&self, e: WindowEvent);
}

#[derive(Debug)]
pub enum WindowEvent {
    CloseWindow,
}

#[derive(Debug)]
pub struct WindowResizeEvent {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct MouseButtonEvent {
    pub down: bool,
    pub button: MouseButton,
}

#[derive(Debug)]
pub struct MouseMoveEvent {
    pub x_pos: u32,
    pub y_pos: u32,
}

#[derive(Debug)]
pub struct MouseWheelEvent {
    pub direction: MouseWheelDirection,
}

#[derive(Debug)]
pub struct KeyEvent {
    pub key: Key,
    pub down: bool,
    pub repeat: u32,
}
