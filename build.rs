use std::env;

#[cfg(target_os = "windows")]
extern crate winres;

fn main() {
    // let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap_or_else(|e| panic!("{}", e));
    let is_android = cfg!(target_os = "android") || (cfg!(target_os = "linux") && target.contains("android")); // hack

    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/sue.ico");
        res.compile().unwrap();
    }

    if target.contains("darwin") {
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.15");
        println!("cargo:rustc-link-arg=-weak_framework");
        println!("cargo:rustc-link-arg=GameController");
        println!("cargo:rustc-link-arg=-weak_framework");
        println!("cargo:rustc-link-arg=CoreHaptics");
    }

    if is_android {
        println!("cargo:rustc-link-lib=dylib=GLESv2");
        println!("cargo:rustc-link-lib=dylib=EGL");
    }
}
