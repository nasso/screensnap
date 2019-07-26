use num_traits::Num;

#[cfg_attr(windows, path = "windows.rs")]
mod os;

#[derive(Debug)]
pub struct Screenshot {
    os: os::OsScreenshot,

    pub data: Vec<u8>,
    pub dimensions: (u32, u32),
    pub windows: Vec<Window>,
}

#[derive(Debug)]
pub struct Window {
    pub bounds: Rectangle<u32>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rectangle<T: Num> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T: Num + Copy> Rectangle<T> {
    pub fn contains<U: PartialOrd<T>>(&self, x: U, y: U) -> bool {
        x >= self.x && y >= self.y && x <= (self.x + self.w) && y <= (self.y + self.h)
    }
}
