#[cfg(target_os = "android")]
#[cfg_attr(target_os = "android", ndk_glue::main())]
pub fn android_main() {
    doukutsu_rs::init().unwrap();
}