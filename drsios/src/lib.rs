#[no_mangle]
pub extern "C" fn drs_main() {
    let options = doukutsu_rs::game::LaunchOptions { server_mode: false, editor: false };

    doukutsu_rs::game::init(options).unwrap();
}
