use std::ops::Add;

#[cfg_attr(windows, path = "windows.rs")]
mod os;

#[derive(Debug)]
pub struct Screenshot {
    os: os::OsScreenshot,

    pub data: Vec<u8>,
    pub dimensions: (u32, u32),
    pub windows: Vec<Window>,
}

impl Screenshot {
    pub fn take() -> Self {
        os::take_screenshot()
    }

    pub fn copy_to_clipboard(&self, region: Rectangle<u32>) {
        os::copy_to_clipboard(self, region);
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Window {
    pub bounds: Rectangle<u32>,
    pub content_bounds: Rectangle<u32>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rectangle<T: PartialEq + PartialOrd + Add<Output = T> + Copy + Clone> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T: PartialEq + PartialOrd + Add<Output = T> + Copy + Clone> Rectangle<T> {
    pub fn contains<U: PartialOrd<T>>(&self, x: U, y: U) -> bool {
        x >= self.x && y >= self.y && x <= (self.x + self.w) && y <= (self.y + self.h)
    }
}
