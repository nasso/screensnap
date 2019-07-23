pub struct Image {
    pub data: Vec<u8>,
    pub dimensions: (u32, u32),
}

#[cfg_attr(windows, path = "windows.rs")]
mod os;

pub use os::snap;
