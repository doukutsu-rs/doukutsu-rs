//#![feature(restricted_std)]

#[repr(C)]
pub struct PrintConsole {}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ApmCpuBoostMode {
    Normal = 0,
    FastLoad = 1,
}

extern "C" {
    fn consoleInit(unk: *mut PrintConsole) -> *mut PrintConsole;
    fn consoleUpdate(unk: *mut PrintConsole);

    fn socketInitialize(unk: *const std::ffi::c_void) -> u32;
    fn nxlinkConnectToHost(redir_stdout: bool, redir_stderr: bool) -> i32;

    fn appletSetCpuBoostMode(mode: ApmCpuBoostMode) -> u32;

    static __text_start: u32;
}

fn main() {
    unsafe {
        // if socketInitialize(std::ptr::null()) == 0 {
        //     nxlinkConnectToHost(true, true);
        // }

        // appletSetCpuBoostMode(ApmCpuBoostMode::FastLoad);

        std::env::set_var("RUST_BACKTRACE", "full");

        println!("__text_start = {:#x}", (&__text_start) as *const _ as usize);

        let options = doukutsu_rs::game::LaunchOptions { server_mode: false, editor: false };
        let result = doukutsu_rs::game::init(options);

        if let Err(e) = result {
            println!("Initialization error: {}", e);
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}
