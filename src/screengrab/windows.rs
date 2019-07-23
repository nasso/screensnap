use super::{Bounds, Screenshot};

use std::{mem::size_of, ptr::null_mut};
use winapi::{
    ctypes::c_void,
    shared::windef::{HBITMAP, HDC},
    um::{
        wingdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
            SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
        },
        winuser::{
            GetDC, GetSystemMetrics, ReleaseDC, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
            SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
        },
    },
};

// take a screenshot
pub fn snap(bounds: Bounds) -> impl Screenshot {
    let (x, y, w, h) = match bounds {
        Bounds::FullScreen => get_full_screen_bounds(),
        Bounds::Area(x, y, w, h) => (x, y, w, h),
    };

    ScreenshotImpl::new(x as i32, y as i32, w as i32, h as i32)
}

// get the combined size of the all the monitors
fn get_full_screen_bounds() -> (u32, u32, u32, u32) {
    return (
        unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) } as u32,
        unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) } as u32,
        unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) } as u32,
        unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) } as u32,
    );
}

// screenshot taking implementation
#[derive(Debug)]
struct ScreenshotImpl {
    h_bitmap: HBITMAPWrapper,
    h_screen: HDCReleaseWrapper,
    h_dc: HDCWrapper,

    rect: (i32, i32, i32, i32),

    data: Vec<u8>,
}

impl Screenshot for ScreenshotImpl {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.rect.2 as u32, self.rect.3 as u32)
    }
}

impl ScreenshotImpl {
    fn new(x: i32, y: i32, w: i32, h: i32) -> ScreenshotImpl {
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

        ScreenshotImpl {
            h_bitmap: h_bitmap.into(),
            h_screen: h_screen.into(),
            h_dc: h_dc.into(),

            rect: (x, y, w, h),

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