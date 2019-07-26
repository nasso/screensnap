use super::{Rectangle, Screenshot, Window};

use std::{
    ffi::OsString,
    mem::{size_of, zeroed},
    os::windows::prelude::*,
    ptr::null_mut,
};
use winapi::{
    ctypes::c_void,
    shared::minwindef::{BOOL, LPARAM},
    shared::windef::{HBITMAP, HDC, HWND, RECT},
    um::{
        dwmapi::{
            DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DWM_CLOAKED_SHELL,
        },
        wingdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
            SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
        },
        winuser::{
            CloseClipboard, EmptyClipboard, EnumWindows, GetDC, GetSystemMetrics, GetWindowInfo,
            GetWindowTextW, IsWindowVisible, OpenClipboard, ReleaseDC, SetClipboardData, CF_BITMAP,
            SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
            WINDOWINFO, WS_POPUP,
        },
    },
};

// os-specific data
#[derive(Debug)]
pub struct OsScreenshot {
    h_bitmap: HBITMAPWrapper,
    h_screen: HDCReleaseWrapper,
    h_dc: HDCWrapper,
}

impl Screenshot {
    pub fn take() -> Self {
        // get virtual screen bounds (covers all monitors)
        let x = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
        let y = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
        let w = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        let h = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };

        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: h,
                biPlanes: 1,
                biBitCount: 24,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: unsafe { std::mem::uninitialized() },
        };

        let h_screen = unsafe { GetDC(null_mut()) };
        let h_dc = unsafe { CreateCompatibleDC(h_screen) };
        let h_bitmap = unsafe { CreateCompatibleBitmap(h_screen, w, h) };

        let mut data = Vec::with_capacity((w * h * 3) as usize);

        unsafe {
            let old_obj = SelectObject(h_dc, h_bitmap as *mut c_void);

            // Get pixels from the screen
            BitBlt(h_dc, 0, 0, w, h, h_screen, x, y, SRCCOPY);
            GetDIBits(
                h_screen,
                h_bitmap,
                0,
                h as u32,
                data.as_mut_ptr() as *mut c_void,
                &mut bi,
                DIB_RGB_COLORS,
            );

            data.set_len(data.capacity());
            SelectObject(h_dc, old_obj);
        }

        // BGR => RGB
        for pixel in data.chunks_exact_mut(3) {
            pixel.swap(0, 2);
        }

        // get all windows now
        struct ProcCallbackData {
            x: i32,
            y: i32,
            windows: Vec<Window>,
        }

        let mut callback_data = ProcCallbackData {
            x,
            y,
            windows: Vec::new(),
        };

        // function that iterates over windows
        pub extern "system" fn enum_windows_proc_callback(wnd: HWND, p: LPARAM) -> BOOL {
            if unsafe { IsWindowVisible(wnd) } != 0 {
                let mut info: WINDOWINFO = unsafe { zeroed() };
                let mut bounds: RECT = unsafe { zeroed() };
                let mut cloaked = 0u32;
                let mut title: Vec<u16> = Vec::with_capacity(128);

                info.cbSize = size_of::<WINDOWINFO>() as u32;

                unsafe {
                    GetWindowInfo(wnd, &mut info);
                    DwmGetWindowAttribute(
                        wnd,
                        DWMWA_EXTENDED_FRAME_BOUNDS,
                        &mut bounds as *mut _ as *mut c_void,
                        size_of::<RECT>() as u32,
                    );
                    DwmGetWindowAttribute(
                        wnd,
                        DWMWA_CLOAKED,
                        &mut cloaked as *mut u32 as *mut c_void,
                        size_of::<u32>() as u32,
                    );
                    let len = GetWindowTextW(wnd, title.as_mut_ptr(), 128);
                    title.set_len(len as usize);
                }

                let title = match OsString::from_wide(&title[..]).into_string() {
                    Ok(s) => s,
                    _ => String::new(),
                };

                // ignore WS_POPUP windows and windows
                if info.dwStyle & WS_POPUP == 0 && cloaked & DWM_CLOAKED_SHELL == 0 {
                    let callback_data = unsafe { (p as *mut ProcCallbackData).as_mut() }.unwrap();

                    callback_data.windows.push(Window {
                        title,
                        bounds: Rectangle {
                            x: bounds.left - callback_data.x,
                            y: bounds.top - callback_data.y,
                            w: (bounds.right - bounds.left),
                            h: (bounds.bottom - bounds.top),
                        },
                    });
                }
            }

            1
        }

        unsafe {
            EnumWindows(
                Some(enum_windows_proc_callback),
                &mut callback_data as *mut ProcCallbackData as LPARAM,
            );
        }

        Screenshot {
            os: OsScreenshot {
                h_bitmap: h_bitmap.into(),
                h_screen: h_screen.into(),
                h_dc: h_dc.into(),
            },

            bounds: Rectangle { x, y, w, h },
            windows: callback_data.windows,
            data,
        }
    }

    pub fn copy_to_clipboard(&self, region: Rectangle<u32>) {
        unsafe {
            let crop = CreateCompatibleBitmap(self.os.h_screen.0, region.w as i32, region.h as i32);
            let h_dc = CreateCompatibleDC(self.os.h_screen.0);

            let old_obj = SelectObject(h_dc, crop as *mut c_void);
            let old_obj_src = SelectObject(self.os.h_dc.0, self.os.h_bitmap.0 as *mut c_void);

            BitBlt(
                h_dc,
                0,
                0,
                region.w as i32,
                region.h as i32,
                self.os.h_dc.0,
                region.x as i32,
                region.y as i32,
                SRCCOPY,
            );

            SelectObject(h_dc, old_obj);
            SelectObject(self.os.h_dc.0, old_obj_src);

            OpenClipboard(null_mut());
            EmptyClipboard();
            SetClipboardData(CF_BITMAP, crop as *mut c_void);
            CloseClipboard();

            DeleteDC(h_dc);
            DeleteObject(crop as *mut c_void);
        }
    }
}

#[derive(Debug)]
struct HBITMAPWrapper(HBITMAP);

impl From<HBITMAP> for HBITMAPWrapper {
    fn from(s: HBITMAP) -> HBITMAPWrapper {
        HBITMAPWrapper(s)
    }
}

impl Drop for HBITMAPWrapper {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.0 as *mut c_void);
        }
    }
}

#[derive(Debug)]
struct HDCWrapper(HDC);

impl From<HDC> for HDCWrapper {
    fn from(s: HDC) -> HDCWrapper {
        HDCWrapper(s)
    }
}

impl Drop for HDCWrapper {
    fn drop(&mut self) {
        unsafe {
            DeleteDC(self.0);
        }
    }
}

#[derive(Debug)]
struct HDCReleaseWrapper(HDC);

impl From<HDC> for HDCReleaseWrapper {
    fn from(s: HDC) -> HDCReleaseWrapper {
        HDCReleaseWrapper(s)
    }
}

impl Drop for HDCReleaseWrapper {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(null_mut(), self.0);
        }
    }
}
