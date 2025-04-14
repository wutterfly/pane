#[derive(Debug)]
pub enum Error {
    RegisterWindowClass(std::io::Error),
    CreateWindow(std::io::Error),
    ShowWindow(std::io::Error),
    SetTitle(std::io::Error),
}

impl Error {
    #[inline]
    #[must_use]
    pub const fn register_window_class(err: std::io::Error) -> Self {
        Self::RegisterWindowClass(err)
    }

    #[inline]
    #[must_use]
    pub const fn create_window(err: std::io::Error) -> Self {
        Self::CreateWindow(err)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RegisterWindowClass(err) => write!(f, "Failed to register window class: {err}"),
            Self::CreateWindow(err) => write!(f, "Failed to create window: {err}"),
            Self::ShowWindow(err) => write!(f, "Failed to show window:  {err}"),
            Self::SetTitle(err) => write!(f, "Failed to set title: {err}"),
        }
    }
}
