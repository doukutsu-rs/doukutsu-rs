use std::os::fd::{FromRawFd, RawFd};

use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{BufRead, BufReader};

extern "C" {
    fn SDL_AndroidGetJNIEnv() -> *mut std::ffi::c_void;
    fn SDL_AndroidGetActivity() -> *mut std::ffi::c_void;
}

#[no_mangle]
pub extern "C" fn SDL_main(_argc: i32, _argv: *const *const u8) -> i32 {
    // From: https://github.com/rust-mobile/ndk-glue/blob/c64e6a47e035dc0869bd2c8b073eae73324e5061/ndk-glue/src/lib.rs#L258-L277
    // Redirect stdout/stderr to logcat, so panic messages get reported.
    unsafe {
        let mut logpipe: [RawFd; 2] = Default::default();
        libc::pipe(logpipe.as_mut_ptr());
        libc::dup2(logpipe[1], libc::STDOUT_FILENO);
        libc::dup2(logpipe[1], libc::STDERR_FILENO);
        std::thread::spawn(move || {
            let tag = CStr::from_bytes_with_nul_unchecked(b"RustStdoutStderr\0");
            let file = File::from_raw_fd(logpipe[0]);
            let mut reader = BufReader::new(file);
            let mut buffer = String::new();
            loop {
                buffer.clear();
                if let Ok(len) = reader.read_line(&mut buffer) {
                    if len == 0 {
                        break;
                    } else if let Ok(msg) = CString::new(buffer.clone()) {
                        ndk_sys::__android_log_write(ndk_sys::android_LogPriority::ANDROID_LOG_INFO.0 as libc::c_int, tag.as_ptr(), msg.as_ptr());
                    }
                }
            }
        });
    }

    android_main();

    unsafe { libc::exit(0) }
}

fn android_main() {
    let env = unsafe { SDL_AndroidGetJNIEnv() };
    let env = unsafe { jni::JNIEnv::from_raw(env.cast()) }.unwrap();
    let vm = env.get_java_vm().unwrap();
    let activity = unsafe { SDL_AndroidGetActivity() };

    unsafe { ndk_context::initialize_android_context(vm.get_java_vm_pointer().cast(), activity) };

    drsandroid_main();

    unsafe { ndk_context::release_android_context(); }
}

fn drsandroid_main() {
    let resource_dir = std::path::PathBuf::from(sdl2::filesystem::pref_path(doukutsu_rs::common::ORG_NAME, doukutsu_rs::common::APP_NAME).unwrap());

    std::env::set_current_dir(&resource_dir).unwrap();
    
    let options = doukutsu_rs::game::LaunchOptions::default();

    doukutsu_rs::game::init(options).unwrap();
}
