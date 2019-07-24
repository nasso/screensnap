use std::{mem::MaybeUninit, ptr::null_mut};

use winapi::um::winuser::{GetMessageW, RegisterHotKey, VK_SNAPSHOT, WM_HOTKEY};

pub fn register<T>(mut callback: T)
where
    T: FnMut() -> bool,
{
    let mut msg = unsafe { MaybeUninit::uninit().assume_init() };

    unsafe {
        RegisterHotKey(null_mut(), 1, 0, VK_SNAPSHOT as u32);
    }

    while unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) } != 0 {
        if msg.message == WM_HOTKEY {
            if !callback() {
                break;
            }
        }
    }
}
