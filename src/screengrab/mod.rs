#[cfg_attr(windows, path = "windows.rs")]
mod os;

pub use os::snap;

pub trait Screenshot {
    fn data(&self) -> &[u8];
    fn dimensions(&self) -> (u32, u32);
    fn windows(&self) -> &[Window];
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rectangle {
    pub fn contains<T: PartialOrd<u32>>(&self, x: T, y: T) -> bool {
        x >= self.x && y >= self.y && x <= (self.x + self.w) && y <= (self.y + self.h)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Bounds {
    FullScreen,
    Area(Rectangle),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Window {
    pub bounds: Rectangle,
    pub content_bounds: Rectangle,
}
