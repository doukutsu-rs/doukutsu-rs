use std::path::PathBuf;

use crate::{
    data::builtin_fs::BuiltinFS,
    framework::{
        context::Context,
        error::GameResult,
        filesystem::{mount_user_vfs, mount_vfs, unmount_user_vfs},
        vfs::PhysicalFS,
    },
};

pub struct FilesystemContainer {
    pub user_path: PathBuf,
    pub game_path: PathBuf,

    pub is_portable: bool,
}

impl FilesystemContainer {
    pub fn new() -> Self {
        Self { user_path: PathBuf::new(), game_path: PathBuf::new(), is_portable: false }
    }

    pub fn mount_fs(&mut self, context: &mut Context) -> GameResult {
        #[cfg(not(any(target_os = "android", target_os = "horizon")))]
        let resource_dir = if let Ok(data_dir) = std::env::var("CAVESTORY_DATA_DIR") {
            PathBuf::from(data_dir)
        } else {
            let mut resource_dir = std::env::current_exe()?;
            if resource_dir.file_name().is_some() {
                let _ = resource_dir.pop();
            }

            #[cfg(target_os = "macos")]
            {
                let mut bundle_dir = resource_dir.clone();
                let _ = bundle_dir.pop();
                let mut bundle_exec_dir = bundle_dir.clone();
                let mut csplus_data_dir = bundle_dir.clone();
                let _ = csplus_data_dir.pop();
                let _ = csplus_data_dir.pop();
                let mut csplus_data_base_dir = csplus_data_dir.clone();
                csplus_data_base_dir.push("data");
                csplus_data_base_dir.push("base");

                bundle_exec_dir.push("MacOS");
                bundle_dir.push("Resources");

                if bundle_exec_dir.is_dir() && bundle_dir.is_dir() {
                    log::info!("Running in macOS bundle mode");

                    if csplus_data_base_dir.is_dir() {
                        log::info!("Cave Story+ Steam detected");
                        resource_dir = csplus_data_dir;
                    } else {
                        resource_dir = bundle_dir;
                    }
                }
            }

            resource_dir.push("data");
            resource_dir
        };

        #[cfg(not(any(target_os = "android", target_os = "horizon")))]
        log::info!("Resource directory: {:?}", resource_dir);

        log::info!("Initializing engine...");

        #[cfg(not(any(target_os = "android", target_os = "horizon")))]
        {
            mount_vfs(context, Box::new(PhysicalFS::new(&resource_dir, true)));
            self.game_path = resource_dir.clone();
        }

        #[cfg(not(any(target_os = "android", target_os = "horizon")))]
        let project_dirs = match directories::ProjectDirs::from("", "", "doukutsu-rs") {
            Some(dirs) => dirs,
            None => {
                use crate::framework::error::GameError;
                return Err(GameError::FilesystemError(String::from(
                    "No valid home directory path could be retrieved.",
                )));
            }
        };
        #[cfg(target_os = "android")]
        {
            let mut data_path =
                PathBuf::from(ndk_glue::native_activity().internal_data_path().to_string_lossy().to_string());
            let mut user_path = data_path.clone();

            data_path.push("data");
            user_path.push("saves");

            let _ = std::fs::create_dir_all(&data_path);
            let _ = std::fs::create_dir_all(&user_path);

            log::info!("Android data directories: data_path={:?} user_path={:?}", &data_path, &user_path);

            mount_vfs(context, Box::new(PhysicalFS::new(&data_path, true)));
            mount_user_vfs(context, Box::new(PhysicalFS::new(&user_path, false)));

            self.user_path = user_path.clone();
            self.game_path = data_path.clone();
        }
        #[cfg(target_os = "horizon")]
        {
            let mut data_path = PathBuf::from("sdmc:/switch/doukutsu-rs/data");
            let mut user_path = PathBuf::from("sdmc:/switch/doukutsu-rs/user");

            let _ = std::fs::create_dir_all(&data_path);
            let _ = std::fs::create_dir_all(&user_path);

            log::info!("Mounting VFS");
            mount_vfs(context, Box::new(PhysicalFS::new(&data_path, true)));
            if crate::framework::backend_horizon::mount_romfs() {
                mount_vfs(context, Box::new(PhysicalFS::new_lowercase(&PathBuf::from("romfs:/data"))));
            }
            log::info!("Mounting user VFS");
            mount_user_vfs(context, Box::new(PhysicalFS::new(&user_path, false)));
            log::info!("ok");

            self.user_path = user_path.clone();
            self.game_path = data_path.clone();
        }

        #[cfg(not(any(target_os = "android", target_os = "horizon")))]
        {
            let mut user_dir = resource_dir.clone();
            user_dir.pop();
            user_dir.push("user");

            if user_dir.is_dir() {
                // portable mode
                mount_user_vfs(context, Box::new(PhysicalFS::new(&user_dir, false)));
                self.user_path = user_dir.clone();
                self.is_portable = true;
            } else {
                let user_dir = project_dirs.data_local_dir();
                mount_user_vfs(context, Box::new(PhysicalFS::new(user_dir, false)));

                self.user_path = user_dir.to_path_buf();
            }
        }

        log::info!("Mounting built-in FS");
        mount_vfs(context, Box::new(BuiltinFS::new()));

        Ok(())
    }

    pub fn open_user_directory(&self) -> GameResult {
        self.open_directory(self.user_path.clone())
    }

    pub fn open_game_directory(&self) -> GameResult {
        self.open_directory(self.game_path.clone())
    }

    pub fn make_portable_user_directory(&mut self, ctx: &mut Context) -> GameResult {
        let mut user_dir = self.game_path.clone();
        user_dir.pop();
        user_dir.push("user");

        if user_dir.is_dir() {
            return Ok(()); // portable directory already exists
        }

        let _ = std::fs::create_dir_all(user_dir.clone());

        // copy user data from current user dir
        for entry in std::fs::read_dir(&self.user_path)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let mut new_path = user_dir.clone();
            new_path.push(file_name);
            std::fs::copy(path, new_path)?;
        }

        // unmount old user dir
        unmount_user_vfs(ctx, &self.user_path);

        // mount new user dir
        mount_user_vfs(ctx, Box::new(PhysicalFS::new(&user_dir, false)));

        self.user_path = user_dir.clone();
        self.is_portable = true;

        Ok(())
    }

    fn open_directory(&self, path: PathBuf) -> GameResult {
        #[cfg(target_os = "horizon")]
        return Ok(()); // can't open directories on switch

        #[cfg(target_os = "android")]
        unsafe {
            use jni::objects::{JObject, JValue};
            use jni::JavaVM;

            let vm_ptr = ndk_glue::native_activity().vm();
            let vm = JavaVM::from_raw(vm_ptr)?;
            let vm_env = vm.attach_current_thread()?;

            let class = vm_env.new_global_ref(JObject::from_raw(ndk_glue::native_activity().activity()))?;
            let method = vm_env.call_method(class.as_obj(), "openDir", "(Ljava/lang/String;)V", &[
                JValue::from(vm_env.new_string(path.to_str().unwrap()).unwrap())
            ])?;

            return Ok(());
        }

        #[cfg(not(any(target_os = "android", target_os = "horizon")))]
        open::that(path).map_err(|e| {
            use crate::framework::error::GameError;
            GameError::FilesystemError(format!("Failed to open directory: {}", e))
        })
    }
}
