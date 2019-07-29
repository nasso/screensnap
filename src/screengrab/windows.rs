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
            CloseClipboard, EmptyClipboard, EnumWindows, GetAncestor, GetDC, GetLastActivePopup,
            GetSystemMetrics, GetTitleBarInfo, GetWindowTextW, IsIconic, IsWindowVisible,
            OpenClipboard, ReleaseDC, SetClipboardData, CF_BITMAP, GA_ROOTOWNER,
            SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
            STATE_SYSTEM_INVISIBLE, TITLEBARINFO,
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

        // Add a newline
        if cfg!(debug_assertions) {
            println!("================================");
            println!("Virtual screen bounds: {}, {}, {}, {}", x, y, w, h);
        }

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
            // ignore invisible windows
            if unsafe { IsWindowVisible(wnd) } == 0 {
                return 1;
            }

            // ignore minimized windows
            if unsafe { IsIconic(wnd) } != 0 {
                return 1;
            }

            // ignore windows in other virtual desktops
            unsafe {
                let mut cloaked = 0u32;

                DwmGetWindowAttribute(
                    wnd,
                    DWMWA_CLOAKED,
                    &mut cloaked as *mut u32 as *mut c_void,
                    size_of::<u32>() as u32,
                );

                // windows in other virtual desktops have the DWM_CLOAKED_SHELL bit set
                if cloaked & DWM_CLOAKED_SHELL != 0 {
                    return 1;
                }
            }

            // https://stackoverflow.com/questions/7277366
            {
                let mut wnd_walk = null_mut();

                // Start at the root owner
                let mut wnd_try = unsafe { GetAncestor(wnd, GA_ROOTOWNER) };

                // See if we are the last active visible popup
                while wnd_try != wnd_walk {
                    wnd_walk = wnd_try;
                    wnd_try = unsafe { GetLastActivePopup(wnd_walk) };

                    if unsafe { IsWindowVisible(wnd_try) } != 0 {
                        break;
                    }
                }

                if wnd_walk != wnd {
                    return 1;
                }

                // remove task tray programs and "Program Manager"
                let mut ti: TITLEBARINFO = unsafe { zeroed() };
                ti.cbSize = size_of::<TITLEBARINFO>() as u32;

                unsafe {
                    GetTitleBarInfo(wnd, &mut ti);
                }

                if ti.rgstate[0] & STATE_SYSTEM_INVISIBLE != 0 {
                    return 1;
                }
            }

            // get the window title
            let title = unsafe {
                let mut buf = Vec::with_capacity(128);

                let len = GetWindowTextW(wnd, buf.as_mut_ptr(), 128);
                buf.set_len(len as usize);

                match OsString::from_wide(&buf[..]).into_string() {
                    Ok(s) => s,
                    _ => String::new(),
                }
            };

            // get the window bounds
            let bounds = unsafe {
                let mut rect: RECT = zeroed();

                DwmGetWindowAttribute(
                    wnd,
                    DWMWA_EXTENDED_FRAME_BOUNDS,
                    &mut rect as *mut _ as *mut c_void,
                    size_of::<RECT>() as u32,
                );

                rect
            };

            // get the ProcCallbackData struct from the pointer
            let callback_data = unsafe { (p as *mut ProcCallbackData).as_mut() }.unwrap();

            // print information about it for debug purposes
            if cfg!(debug_assertions) {
                println!("Window {:?}:", wnd);
                println!("  Title: {}", title);
                println!(
                    "  Bounds: {:?}",
                    Rectangle {
                        x: bounds.left - callback_data.x,
                        y: bounds.top - callback_data.y,
                        w: (bounds.right - bounds.left),
                        h: (bounds.bottom - bounds.top),
                    }
                );
            }

            // add the window to the list
            callback_data.windows.push(Window {
                title,
                bounds: Rectangle {
                    x: bounds.left - callback_data.x,
                    y: bounds.top - callback_data.y,
                    w: (bounds.right - bounds.left),
                    h: (bounds.bottom - bounds.top),
                },
            });

            // return 1 to get more windows!
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
