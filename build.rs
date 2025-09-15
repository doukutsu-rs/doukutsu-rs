use std::env;

#[cfg(target_os = "windows")]
extern crate winres;

fn main() {
    let target = env::var("TARGET").unwrap_or_else(|e| panic!("{}", e));

    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "windows")]
    if target.contains("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/crabsue-icon.ico");
        res.compile().unwrap();

        if target.contains("i686") {
            // hack
            println!("cargo:rustc-link-arg=/FORCE:MULTIPLE");
            println!("cargo:rustc-link-lib=shlwapi");
        }
    }

    if target.contains("darwin") {
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.15");
        println!("cargo:rustc-link-arg=-weak_framework");
        println!("cargo:rustc-link-arg=GameController");
        println!("cargo:rustc-link-arg=-weak_framework");
        println!("cargo:rustc-link-arg=CoreHaptics");
    }

    if target.contains("android") {
        println!("cargo:rustc-link-lib=dylib=GLESv2");
        println!("cargo:rustc-link-lib=dylib=EGL");
    }
}
