use std::env;

// #[cfg(feature = "generate-gl")]
// use gl_generator::{Api, Fallbacks, Profile, Registry};

fn main() {
    // let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap_or_else(|e| panic!("{}", e));
    let is_android = cfg!(target_os = "android") || (cfg!(target_os = "linux") && target.contains("android")); // hack

    println!("cargo:rerun-if-changed=build.rs");
    //
    // #[cfg(feature = "generate-gl")]
    // {
    //     let mut file = File::create(&dest.join("gl_bindings.rs")).unwrap();
    //
    //     Registry::new(Api::Gles2, (3, 0), Profile::Core, Fallbacks::All, [])
    //         .write_bindings(gl_generator::StructGenerator, &mut file)
    //         .unwrap();
    // }

    if is_android {
        println!("cargo:rustc-link-lib=dylib=GLESv2");
        println!("cargo:rustc-link-lib=dylib=EGL");
    }
}
