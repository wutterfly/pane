use std::{ffi::CString, sync::Arc};

use x11rb::{
    COPY_DEPTH_FROM_PARENT,
    connection::Connection as _,
    protocol::xproto::{
        AtomEnum, ConnectionExt, CreateWindowAux, EventMask, PropMode, WindowClass,
    },
    wrapper::ConnectionExt as _,
    xcb_ffi::XCBConnection,
};

use crate::WindowImpl;
use crate::events::{KeyEvent, MouseButtonEvent, MouseMoveEvent, WindowEvent, WindowResizeEvent};

x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
    }
}

#[derive(Debug)]
pub struct Window {
    conn: XCBConnection,
    window: u32,
    eventsys: Arc<dyn crate::events::EventSystem>,
    atoms: Atoms,
}

impl WindowImpl for Window {
    fn create(
        title: &str,
        eventsys: Arc<dyn crate::events::EventSystem>,
        window_pos_x: i16,
        window_pos_y: i16,
        window_width: u16,
        window_height: u16,
    ) -> Result<Self, super::Error> {
        // use raw xcb connection instead of rust connection to interface with vulkan
        let (conn, screen_num) = x11rb::xcb_ffi::XCBConnection::connect(None)?;

        let screen = &conn.setup().roots[screen_num];
        let window = conn.generate_id()?;

        // load atoms
        let atoms = Atoms::new(&conn)?.reply()?;

        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            screen.root,
            window_pos_x,
            window_pos_y,
            window_width,
            window_height,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new()
                .background_pixel(screen.black_pixel)
                .event_mask(
                    EventMask::EXPOSURE
                        | EventMask::KEY_PRESS
                        | EventMask::KEY_RELEASE
                        | EventMask::BUTTON_PRESS
                        | EventMask::BUTTON_RELEASE
                        | EventMask::POINTER_MOTION
                        | EventMask::STRUCTURE_NOTIFY,
                ),
        )?;

        // set title
        conn.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            title.as_bytes(),
        )?;

        conn.change_property32(
            PropMode::REPLACE,
            window,
            atoms.WM_PROTOCOLS,
            AtomEnum::ATOM,
            &[atoms.WM_DELETE_WINDOW],
        )?;

        Ok(Self {
            conn,
            window,
            eventsys,
            atoms,
        })
    }

    fn show(&self) -> Result<(), super::Error> {
        self.conn.map_window(self.window)?;
        self.conn.flush()?;
        Ok(())
    }

    fn set_title(&self, title: &str) -> Result<(), super::Error> {
        let title = CString::new(title).unwrap();

        self.conn.change_property8(
            PropMode::REPLACE,
            self.window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            title.as_bytes(),
        )?;

        self.conn.flush()?;

        Ok(())
    }

    fn destroy(self) {
        _ = self.conn.destroy_window(self.window);
    }

    fn pump_messages(&mut self) -> Result<(), super::Error> {
        loop {
            let Some(event) = self.conn.poll_for_event()? else {
                return Ok(());
            };

            match &event {
                // key events
                x11rb::protocol::Event::KeyPress(k) | x11rb::protocol::Event::KeyRelease(k) => {
                    let is_pressed = matches!(event, x11rb::protocol::Event::KeyPress(_));
                    let raw_key = k.detail;

                    let key = super::inputs::x_translate_key(raw_key);

                    self.eventsys.invoke_key_event(KeyEvent {
                        key,
                        down: is_pressed,
                        repeat: 0,
                    });
                }

                // mouse button events
                x11rb::protocol::Event::ButtonPress(e)
                | x11rb::protocol::Event::ButtonRelease(e) => {
                    let is_pressed = matches!(event, x11rb::protocol::Event::ButtonPress(_));
                    let raw_key = e.detail;

                    let button = super::inputs::x_translate_button(raw_key);

                    self.eventsys.invoke_mouse_button_event(MouseButtonEvent {
                        down: is_pressed,
                        button,
                    });
                }

                // mouse move
                x11rb::protocol::Event::MotionNotify(m) => {
                    let x = m.event_x;
                    let y = m.event_y;

                    self.eventsys.invoke_mouse_move_event(MouseMoveEvent {
                        x_pos: u32::try_from(x).unwrap(),
                        y_pos: u32::try_from(y).unwrap(),
                    });
                }

                // resize
                x11rb::protocol::Event::ConfigureNotify(r) => {
                    let height = r.height;
                    let width = r.width;

                    self.eventsys.invoke_window_resize_event(WindowResizeEvent {
                        width: u32::from(width),
                        height: u32::from(height),
                    });
                }

                // close window
                x11rb::protocol::Event::ClientMessage(m) => {
                    // We have received a message from the server
                    let atom = m.data.as_data32()[0];

                    if atom == self.atoms.WM_DELETE_WINDOW {
                        self.eventsys.invoke_window_event(WindowEvent::CloseWindow);

                        break Ok(());
                    }
                }

                _ => {}
            }
        }
    }

    fn inner_size(&self) -> crate::Rect {
        let geometry = self
            .conn
            .get_geometry(self.window)
            .unwrap()
            .reply()
            .unwrap();

        crate::Rect::new(geometry.width, geometry.height)
    }

    fn raw_handle(&self) -> crate::RawWindowHandle {
        let raw_conn = self.conn.get_raw_xcb_connection();

        crate::RawWindowHandle::Xcb {
            connection: raw_conn,
            window: self.window,
        }
    }
}
