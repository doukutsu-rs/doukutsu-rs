#[cfg(target_os = "android")]
#[cfg_attr(target_os = "android", ndk_glue::main())]
pub fn android_main() {
    let resource_dir = std::path::PathBuf::from(ndk_glue::native_activity().internal_data_path().to_string_lossy().to_string());

    let _ = std::env::set_current_dir(&resource_dir);

    let options = doukutsu_rs::game::LaunchOptions::default();

    if let Err(e) = doukutsu_rs::game::init(options) {
        eprintln!("Game init failed: {:?}", e);
    }
}
