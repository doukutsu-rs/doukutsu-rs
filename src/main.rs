#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::exit;

fn main() {
    let result = doukutsu_rs::init();

    #[cfg(target_os = "windows")]
        unsafe {
        use winapi::_core::ptr::null_mut;
        use winapi::um::winuser::MessageBoxW;
        use winapi::um::winuser::MB_OK;
        use winapi::shared::ntdef::LPCWSTR;
        use std::ffi::OsStr;
        use std::os::windows::prelude::*;

        if let Err(e) = result {
            let title: LPCWSTR = OsStr::new("Error!")
                .encode_wide().chain(Some(0)).collect::<Vec<u16>>().as_ptr();
            let message: LPCWSTR = OsStr::new(format!("Whoops, nxengine-rs crashed: {}", e).as_str())
                .encode_wide().chain(Some(0)).collect::<Vec<u16>>().as_ptr();
            MessageBoxW(null_mut(),
                        message,
                        title,
                        MB_OK);
            exit(1);
        }
    }

    if let Err(e) = result {
        println!("Initialization error: {}", e);
        exit(1);
    }
}
