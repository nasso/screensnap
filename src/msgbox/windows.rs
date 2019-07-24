use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt, ptr::null_mut};
use winapi::um::winuser::{MessageBoxW, MB_ICONERROR, MB_OK, MB_SYSTEMMODAL};

pub fn error(msg: &str) {
    let wide: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    unsafe {
        MessageBoxW(
            null_mut(),
            wide.as_ptr(),
            null_mut(),
            MB_OK | MB_ICONERROR | MB_SYSTEMMODAL,
        )
    };
}
