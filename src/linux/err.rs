#[derive(Debug)]
pub enum X11Error {
    Connection(x11rb::errors::ConnectionError),
    Id(x11rb::errors::ReplyOrIdError),
    Connect(x11rb::errors::ConnectError),
    Reply(x11rb::errors::ReplyError),
}

impl std::error::Error for X11Error {}

impl std::fmt::Display for X11Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connection(err) => write!(f, "{err}"),
            Self::Connect(err) => write!(f, "{err}"),
            Self::Id(err) => write!(f, "{err}"),
            Self::Reply(err) => write!(f, "{err}"),
        }
    }
}

#[cfg(target_os = "linux")]
impl From<x11rb::errors::ReplyOrIdError> for X11Error {
    fn from(value: x11rb::errors::ReplyOrIdError) -> Self {
        Self::Id(value)
    }
}

#[cfg(target_os = "linux")]
impl From<x11rb::errors::ConnectError> for X11Error {
    fn from(value: x11rb::errors::ConnectError) -> Self {
        Self::Connect(value)
    }
}

#[cfg(target_os = "linux")]
impl From<x11rb::errors::ConnectionError> for X11Error {
    fn from(value: x11rb::errors::ConnectionError) -> Self {
        Self::Connection(value)
    }
}

#[cfg(target_os = "linux")]
impl From<x11rb::errors::ReplyError> for X11Error {
    fn from(value: x11rb::errors::ReplyError) -> Self {
        Self::Reply(value)
    }
}
