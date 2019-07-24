use super::{Rectangle, Screenshot, Window};

use std::{
    mem::{size_of, MaybeUninit},
    ptr::null_mut,
};
use winapi::{
    ctypes::c_void,
    shared::minwindef::{BOOL, LPARAM},
    shared::windef::{HBITMAP, HDC, HWND},
    um::{
        wingdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
            SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
        },
        winuser::{
            CloseClipboard, EmptyClipboard, EnumWindows, GetDC, GetSystemMetrics, GetWindowInfo,
            IsWindowVisible, OpenClipboard, ReleaseDC, SetClipboardData, CF_BITMAP,
            SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
            WINDOWINFO, WS_POPUP,
        },
    },
};

// take a screenshot
pub fn snap() -> impl Screenshot {
    ScreenshotImpl::new()
}

// screenshot taking implementation
#[derive(Debug)]
struct ScreenshotImpl {
    h_bitmap: HBITMAPWrapper,
    h_screen: HDCReleaseWrapper,
    h_dc: HDCWrapper,

    rect: (i32, i32, i32, i32),

    data: Vec<u8>,
    windows: Vec<Window>,
}

impl Screenshot for ScreenshotImpl {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.rect.2 as u32, self.rect.3 as u32)
    }

    fn windows(&self) -> &[Window] {
        &self.windows
    }

    fn copy_to_clipboard(&self, region: Rectangle<u32>) {
        unsafe {
            let crop = CreateCompatibleBitmap(self.h_screen.0, region.w as i32, region.h as i32);
            let h_dc = CreateCompatibleDC(self.h_screen.0);

            let old_obj = SelectObject(h_dc, crop as *mut c_void);
            let old_obj_src = SelectObject(self.h_dc.0, self.h_bitmap.0 as *mut c_void);

            BitBlt(
                h_dc,
                0,
                0,
                region.w as i32,
                region.h as i32,
                self.h_dc.0,
                region.x as i32,
                region.y as i32,
                SRCCOPY,
            );

            SelectObject(h_dc, old_obj);
            SelectObject(self.h_dc.0, old_obj_src);

            OpenClipboard(null_mut());
            EmptyClipboard();
            SetClipboardData(CF_BITMAP, crop as *mut c_void);
            CloseClipboard();

            DeleteDC(h_dc);
            DeleteObject(crop as *mut c_void);
        }
    }
}

impl ScreenshotImpl {
    fn new() -> ScreenshotImpl {
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
        let mut windows = Vec::new();

        // function that iterates over windows
        pub extern "system" fn enum_windows_proc_callback(wnd: HWND, p: LPARAM) -> BOOL {
            if unsafe { IsWindowVisible(wnd) } != 0 {
                let mut info: WINDOWINFO = unsafe { MaybeUninit::uninit().assume_init() };

                info.cbSize = size_of::<WINDOWINFO>() as u32;

                unsafe {
                    GetWindowInfo(wnd, &mut info);
                }

                // ignore WS_POPUP windows
                if info.dwStyle & WS_POPUP == 0 {
                    let windows = unsafe { (p as *mut Vec<_>).as_mut() }.unwrap();

                    windows.push(Window {
                        bounds: Rectangle {
                            x: info.rcWindow.left as u32,
                            y: info.rcWindow.top as u32,
                            w: (info.rcWindow.right - info.rcWindow.left) as u32,
                            h: (info.rcWindow.bottom - info.rcWindow.top) as u32,
                        },

                        content_bounds: Rectangle {
                            x: info.rcClient.left as u32,
                            y: info.rcClient.top as u32,
                            w: (info.rcClient.right - info.rcClient.left) as u32,
                            h: (info.rcClient.bottom - info.rcClient.top) as u32,
                        },
                    });
                }
            }

            1
        }

        unsafe {
            EnumWindows(
                Some(enum_windows_proc_callback),
                &mut windows as *mut Vec<_> as LPARAM,
            );
        }

        ScreenshotImpl {
            h_bitmap: h_bitmap.into(),
            h_screen: h_screen.into(),
            h_dc: h_dc.into(),

            rect: (x, y, w, h),

            windows,
            data,
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
