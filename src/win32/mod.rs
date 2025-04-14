mod err;
mod inputs;
mod userdata;

pub use err::Error;

use std::{
    os::windows::ffi::OsStrExt,
    ptr::{null, null_mut},
    sync::Arc,
};

use windows_sys::{
    Win32::{
        Foundation::{HMODULE, HWND, LPARAM, LRESULT, RECT, SetLastError, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, CS_DBLCLKS, CreateWindowExW, DefWindowProcW, DestroyWindow,
            DispatchMessageW, GWLP_USERDATA, GetClientRect, GetWindowLongPtrW, IDC_ARROW,
            IDI_APPLICATION, LoadCursorW, LoadIconW, MSG, PM_REMOVE, PeekMessageW, PostQuitMessage,
            RegisterClassExW, SW_SHOW, SW_SHOWNOACTIVATE, SetWindowLongPtrW, SetWindowTextW,
            ShowWindow, TranslateMessage, WM_CLOSE, WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN,
            WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE,
            WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SIZE, WM_SYSKEYDOWN, WM_SYSKEYUP,
            WNDCLASSEXW, WS_CAPTION, WS_EX_APPWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX,
            WS_OVERLAPPED, WS_OVERLAPPEDWINDOW, WS_SYSMENU, WS_THICKFRAME,
        },
    },
    w,
};

use crate::{
    RawWindowHandle, Rect, WindowImpl,
    events::{
        self, KeyEvent, MouseButtonEvent, MouseMoveEvent, MouseWheelEvent, WindowResizeEvent,
    },
    inputs::{Key, MouseButton, MouseWheelDirection},
};

use self::userdata::UserData;

#[derive(Debug)]
pub struct Window {
    handle: HWND,
    instance: HMODULE,
}

impl WindowImpl for Window {
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn create(
        title: &str,
        eventsys: Arc<dyn events::EventSystem>,
        window_pos_x: i16,
        window_pos_y: i16,
        window_width: u16,
        window_height: u16,
    ) -> Result<Self, Error> {
        let userdata = UserData::new(eventsys);

        let instance: HMODULE = unsafe { GetModuleHandleW(null()) };

        // create window class
        let class_name = win32_string("Windows_Class");

        let icon = unsafe { LoadIconW(instance, IDI_APPLICATION) };
        let cursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_DBLCLKS,
            lpfnWndProc: Some(process_messages),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: icon,
            hCursor: cursor,
            hbrBackground: null_mut(),
            lpszClassName: class_name.as_ptr(),
            lpszMenuName: w!("Menu name"),
            hIconSm: null_mut(),
        };

        if unsafe { RegisterClassExW(&wnd_class) } == 0 {
            return Err(Error::register_window_class(std::io::Error::last_os_error()))
                .inspect_err(|err| log::error!("{err}"));
        }

        // calculate window sizes
        let window_styles = WS_OVERLAPPED
            | WS_SYSMENU
            | WS_CAPTION
            | WS_MAXIMIZEBOX
            | WS_MINIMIZEBOX
            | WS_THICKFRAME;
        let windows_styles_ex = WS_EX_APPWINDOW;

        let mut window_pos_x = i32::from(window_pos_x);
        let mut window_pos_y = i32::from(window_pos_y);
        let mut window_width = i32::from(window_width);
        let mut window_height = i32::from(window_height);
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        unsafe {
            AdjustWindowRectEx(
                std::ptr::addr_of_mut!(rect),
                window_styles,
                0,
                windows_styles_ex,
            )
        };

        window_pos_x += rect.left;
        window_pos_y += rect.top;
        window_width += rect.right - rect.left;
        window_height += rect.bottom - rect.top;

        let title = win32_string(title);

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                wnd_class.lpszClassName,
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                window_pos_x,
                window_pos_y,
                window_width,
                window_height,
                null_mut(),
                null_mut(),
                instance,
                null_mut(),
            )
        };

        // deallocate "strings"
        drop(class_name);
        drop(title);

        if hwnd.is_null() {
            return Err(Error::create_window(std::io::Error::last_os_error()))
                .inspect_err(|err| log::error!("{err}"));
        }

        let window = Self {
            handle: hwnd,
            instance,
        };

        window.set_user_data(Box::new(userdata));

        Ok(window)
    }

    fn show(&self) -> Result<(), Error> {
        let should_activate = true;
        let flags = if should_activate {
            SW_SHOW
        } else {
            SW_SHOWNOACTIVATE
        };

        // If the window was previously visible, the return value is nonzero.
        // If the window was previously hidden, the return value is zero.
        let _ = unsafe { ShowWindow(self.handle, flags) };

        Ok(())
    }

    fn set_title(&self, title: &str) -> Result<(), Error> {
        let title = win32_string(title);

        let ret = unsafe { SetWindowTextW(self.handle, title.as_ptr()) };

        // If the function succeeds, the return value is nonzero.
        // If the function fails, the return value is zero.
        if ret == 0 {
            let err = std::io::Error::last_os_error();
            return Err(Error::SetTitle(err));
        }

        Ok(())
    }

    fn destroy(self) {
        let userdata = Self::get_user_data(self.handle);
        drop(userdata);

        let res = unsafe { DestroyWindow(self.handle) };
        debug_assert_ne!(res, 0);
    }

    fn pump_messages(&mut self) -> Result<(), Error> {
        let mut msg: MSG = unsafe { std::mem::zeroed() };
        let ptr = std::ptr::addr_of_mut!(msg);

        // SAFETY:
        // PeekMessageW is safe to call, because ptr is a valid pointer,
        // hwnd := 0 retrieves messages from all Windows on the current thread,
        // wMsgFilterMin,wMsgFilterMax := 0 filters no messages
        // wRemoveMsg := 1 removes messages from queue
        while unsafe { PeekMessageW(ptr, self.handle, 0, 0, PM_REMOVE) } != 0 {
            // SAFETY:
            // This is safe to use, because ptr is a valid pointer to a MSG struct.
            unsafe {
                TranslateMessage(ptr);
                DispatchMessageW(ptr)
            };
        }

        Ok(())
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn inner_size(&self) -> Rect {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        let res = unsafe { GetClientRect(self.handle, &mut rect) };

        debug_assert!(res != 0, "Error getting client rect: {res}");

        #[allow(clippy::cast_sign_loss)]
        Rect {
            x: (rect.right - rect.left) as u16,
            y: (rect.bottom - rect.top) as u16,
        }
    }

    #[inline]
    fn raw_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Win32 {
            hwnd: self.handle as _,
            hinstance: self.instance as _,
        }
    }
}

impl Window {
    #[inline]
    fn set_user_data(&self, userdata: Box<UserData>) {
        // reset error
        unsafe { SetLastError(0) };

        let ptr = Box::into_raw(userdata) as isize;

        let res = unsafe { SetWindowLongPtrW(self.handle, GWLP_USERDATA, ptr) };

        assert_eq!(res, 0);
    }

    #[inline]
    fn get_user_data(handle: HWND) -> Option<Box<UserData>> {
        let ptr = unsafe { GetWindowLongPtrW(handle, GWLP_USERDATA) } as *mut UserData;

        if ptr.is_null() {
            return None;
        }

        let userdata = unsafe { Box::from_raw(ptr) };

        Some(userdata)
    }
}

#[inline]
fn win32_string(str: &str) -> Vec<u16> {
    std::ffi::OsStr::new(str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
unsafe extern "system" fn process_messages(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let Some(userdata) = Window::get_user_data(hwnd) else {
            // the first few messages after window creation won't be handled
            log::trace!("failed to load userdata");

            return DefWindowProcW(hwnd, msg, wparam, lparam);
        };

        let callback = || match msg {
            WM_ERASEBKGND => {
                // erasing the screen will be handled by application
                1
            }
            // quit
            WM_CLOSE => {
                userdata
                    .events()
                    .invoke_window_event(events::WindowEvent::CloseWindow);

                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            // resize
            WM_SIZE => {
                let mut rect: RECT = std::mem::zeroed();
                GetClientRect(hwnd, std::ptr::addr_of_mut!(rect));
                let width: u32 = (rect.right - rect.left).try_into().unwrap();
                let height: u32 = (rect.bottom - rect.top).try_into().unwrap();

                userdata
                    .events()
                    .invoke_window_resize_event(WindowResizeEvent { width, height });

                0
            }
            // key up/down
            WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
                // key pressed?
                let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;

                let key = Key::from_win32_key(wparam as u16);

                let repeat = ((lparam & 30) as u32).clamp(0, 1);

                userdata
                    .events()
                    .invoke_key_event(KeyEvent { key, down, repeat });

                0
            }
            // mouse move
            WM_MOUSEMOVE => {
                let x_pos = get_x_lparam(lparam);
                let y_pos = get_y_lparam(lparam);

                userdata
                    .events()
                    .invoke_mouse_move_event(MouseMoveEvent { x_pos, y_pos });

                0
            }
            // mouse wheel
            WM_MOUSEWHEEL => {
                let delta = get_wheel_delta_wparam(wparam);

                // flatten delta to (-1, 1)
                let direction = match delta {
                    z_delta if z_delta < 0 => MouseWheelDirection::Down,
                    z_delta if z_delta > 0 => MouseWheelDirection::Up,
                    _ => return 0,
                };

                userdata
                    .events()
                    .invoke_mouse_wheel_event(MouseWheelEvent { direction });

                0
            }
            // mouse button
            WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN | WM_LBUTTONUP | WM_MBUTTONUP
            | WM_RBUTTONUP => {
                // key pressed?
                let down = msg == WM_LBUTTONDOWN || msg == WM_MBUTTONDOWN || msg == WM_RBUTTONDOWN;

                let button = match msg {
                    WM_LBUTTONDOWN | WM_LBUTTONUP => MouseButton::Left,
                    WM_RBUTTONDOWN | WM_RBUTTONUP => MouseButton::Right,
                    _ => MouseButton::Middle,
                };

                userdata
                    .events()
                    .invoke_mouse_button_event(MouseButtonEvent { down, button });

                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        };

        let res = callback();

        // make sure userdata is not deallocated
        std::mem::forget(userdata);

        res
    }
}

#[inline]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
const fn get_x_lparam(lparam: LPARAM) -> u32 {
    let lower_32_bits = lparam as i32;
    (lower_32_bits & 0xffff) as u32
}

#[inline]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
const fn get_y_lparam(lparam: LPARAM) -> u32 {
    let lower_32_bits = lparam as i32;
    ((lower_32_bits >> 16) & 0xffff) as u32
}

#[inline]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
const fn get_wheel_delta_wparam(wparam: WPARAM) -> i16 {
    (wparam >> 16) as i16
}
