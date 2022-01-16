#[cfg(target_os = "android")]
#[cfg_attr(target_os = "android", ndk_glue::main())]
pub fn android_main() {
    let options = doukutsu_rs::LaunchOptions { server_mode: false, editor: false };

    doukutsu_rs::init(options).unwrap();
}
