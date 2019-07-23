#[cfg_attr(windows, path = "windows.rs")]
mod os;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Bounds {
    FullScreen,
    Area(Rectangle),
}

#[derive(Debug, PartialEq)]
pub struct Window {
    pub bounds: Rectangle,
    pub content_bounds: Rectangle,
}

pub trait Screenshot {
    fn data(&self) -> &[u8];
    fn dimensions(&self) -> (u32, u32);
    fn windows(&self) -> &[Window];
}

pub use os::snap;
