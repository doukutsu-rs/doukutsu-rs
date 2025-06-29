#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::exit;

use clap::Parser;

fn main() {
    let options = doukutsu_rs::game::LaunchOptions::parse();

    if options.server_mode && options.editor {
        eprintln!("Cannot run in server mode and editor mode at the same time.");
        exit(1);
    }

    let result = doukutsu_rs::game::init(options);

    #[cfg(target_os = "windows")]
        unsafe {
        use std::ffi::OsStr;
        use std::os::windows::prelude::*;
        use winapi::_core::ptr::null_mut;
        use winapi::shared::ntdef::LPCWSTR;
        use winapi::um::winuser::MessageBoxW;
        use winapi::um::winuser::MB_OK;

        if let Err(e) = result {
            let title: LPCWSTR = OsStr::new("Error!").encode_wide().chain(Some(0)).collect::<Vec<u16>>().as_ptr();
            let message: LPCWSTR = OsStr::new(format!("Whoops, doukutsu-rs crashed: {}", e).as_str())
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<u16>>()
                .as_ptr();
            MessageBoxW(null_mut(), message, title, MB_OK);
            exit(1);
        }
    }

    if let Err(e) = result {
        eprintln!("Initialization error: {}", e);
        exit(1);
    }
}
