//#![feature(restricted_std)]

#[repr(C)]
pub struct PrintConsole {}

extern "C" {
    pub fn consoleInit(unk: *mut PrintConsole) -> *mut PrintConsole;

    pub fn consoleUpdate(unk: *mut PrintConsole);
}

fn main() {
    unsafe {
        consoleInit(std::ptr::null_mut());

        let options = doukutsu_rs::game::LaunchOptions { server_mode: false, editor: false };
        let result = doukutsu_rs::game::init(options);

        if let Err(e) = result {
            println!("Initialization error: {}", e);
            loop {
                consoleUpdate(std::ptr::null_mut());
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}
