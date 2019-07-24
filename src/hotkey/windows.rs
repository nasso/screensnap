use std::{mem::MaybeUninit, ptr::null_mut};

use winapi::um::winuser::{
    GetMessageW, PeekMessageW, RegisterHotKey, PM_REMOVE, VK_SNAPSHOT, WM_HOTKEY,
};

pub fn register<T>(consume_queue: bool, mut callback: T)
where
    T: FnMut() -> bool,
{
    let mut msg = unsafe { MaybeUninit::uninit().assume_init() };

    unsafe {
        RegisterHotKey(null_mut(), 1, 0, VK_SNAPSHOT as u32);
    }

    while unsafe { GetMessageW(&mut msg, null_mut(), WM_HOTKEY, WM_HOTKEY) } != 0 {
        if !callback() {
            break;
        } else if consume_queue {
            // consume all hotkey events
            while unsafe { PeekMessageW(&mut msg, null_mut(), WM_HOTKEY, WM_HOTKEY, PM_REMOVE) }
                != 0
            {}
        }
    }
}
