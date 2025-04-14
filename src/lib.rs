#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::struct_field_names)]

pub mod events;
pub mod inputs;

use std::{ffi::c_void, sync::Arc};

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "windows")]
pub use win32::Error;
#[cfg(target_os = "windows")]
use win32::Window as TargetWindow;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
use linux::Window as TargetWindow;

#[cfg(target_os = "linux")]
pub use linux::Error;

trait WindowImpl {
    fn create(
        title: &str,
        eventsys: Arc<dyn events::EventSystem>,
        window_pos_x: i16,
        window_pos_y: i16,
        window_width: u16,
        window_height: u16,
    ) -> Result<Self, Error>
    where
        Self: Sized;

    fn show(&self) -> Result<(), Error>;

    fn set_title(&self, title: &str) -> Result<(), Error>;

    fn destroy(self);

    fn pump_messages(&mut self) -> Result<(), Error>;

    fn inner_size(&self) -> Rect;

    fn raw_handle(&self) -> RawWindowHandle;
}

#[derive(Debug)]
pub struct Window {
    window: TargetWindow,
}

impl Window {
    /// Creates a new Window.
    ///
    /// # Errors
    /// Returns an `Error` if creating the window failed (wow :D).
    #[inline]
    pub fn create(
        name: &str,
        eventsys: Arc<dyn events::EventSystem>,
        pos: Rect,
        size: Rect,
    ) -> Result<Self, Error> {
        #[allow(clippy::cast_possible_wrap)]
        let window = <TargetWindow as WindowImpl>::create(
            name,
            eventsys,
            pos.x as i16,
            pos.y as i16,
            size.x,
            size.y,
        )?;

        Ok(Self { window })
    }

    /// Presents the window to the user.
    ///
    /// # Errors
    #[inline]
    pub fn show(&self) -> Result<(), crate::Error> {
        <TargetWindow as WindowImpl>::show(&self.window)
    }

    /// Sets the title of the window.
    ///
    /// # Errors
    #[inline]
    pub fn set_title(&self, title: &str) -> Result<(), crate::Error> {
        <TargetWindow as WindowImpl>::set_title(&self.window, title)
    }

    /// Destroys the window.
    #[inline]
    pub fn destroy(self) {
        self.window.destroy();
    }

    /// Processes window events.
    ///
    /// # Errors
    #[inline]
    pub fn pump_messages(&mut self) -> Result<(), Error> {
        <TargetWindow as WindowImpl>::pump_messages(&mut self.window)
    }

    /// Returns the screen size.
    #[inline]
    #[must_use]
    pub fn inner_size(&self) -> Rect {
        <TargetWindow as WindowImpl>::inner_size(&self.window)
    }

    /// Returns a `RawWindowHandle`,  mostly used by graphics APIs.
    ///
    /// # Error
    #[must_use]
    pub fn raw_handle(&self) -> RawWindowHandle {
        <TargetWindow as WindowImpl>::raw_handle(&self.window)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
}

impl Rect {
    #[inline]
    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawWindowHandle {
    Win32 {
        hwnd: isize,
        hinstance: isize,
    },
    Xcb {
        connection: *mut c_void,
        window: u32,
    },
}
