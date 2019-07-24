use winapi::{
    shared::minwindef::{BOOL, LPARAM, LPDWORD},
    shared::windef::HWND,
    um::winuser::{EnumWindows, GetWindowThreadProcessId, SetForegroundWindow},
};

#[derive(Debug)]
struct Data {
    pid: u32,
    win: HWND,
}

fn get_process_window(pid: u32) -> HWND {
    let mut data = Data {
        pid,
        win: 0 as HWND,
    };

    // function that iterates over windows
    pub extern "system" fn enum_windows_proc_callback(wnd: HWND, p: LPARAM) -> BOOL {
        let mut win_pid = 0;

        unsafe {
            GetWindowThreadProcessId(wnd, &mut win_pid as *mut u32 as LPDWORD);
        }

        let mut data = unsafe { (p as *mut Data).as_mut() }.unwrap();

        if win_pid == data.pid {
            data.win = wnd;

            0
        } else {
            1
        }
    }

    unsafe {
        EnumWindows(
            Some(enum_windows_proc_callback),
            &mut data as *mut Data as LPARAM,
        );
    }

    data.win
}

pub fn focus_current_window() {
    unsafe {
        SetForegroundWindow(get_process_window(std::process::id()));
    }
}
