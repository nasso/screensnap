#[cfg_attr(windows, path = "windows.rs")]
mod os;

pub trait Screenshot {
    fn data(&self) -> &[u8];
    fn dimensions(&self) -> (u32, u32);
}

#[derive(Debug, Copy, Clone)]
pub enum Bounds {
    FullScreen,
    Area(u32, u32, u32, u32),
}

pub use os::snap;
