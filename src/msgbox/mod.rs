#[cfg_attr(windows, path = "windows.rs")]
mod os;

pub use os::error;
