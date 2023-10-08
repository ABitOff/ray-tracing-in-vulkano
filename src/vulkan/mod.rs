pub mod application;
pub mod window;

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub cursor_disabled: bool,
    pub fullscreen: bool,
    pub resizable: bool,
}
