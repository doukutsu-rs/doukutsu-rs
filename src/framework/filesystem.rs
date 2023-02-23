use std::fmt;
use std::io;
use std::io::SeekFrom;
use std::path;
use std::path::PathBuf;

use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::vfs;
use crate::framework::vfs::{OpenOptions, VFS};

/// A structure that contains the filesystem state and cache.
#[derive(Debug)]
pub struct Filesystem {
    vfs: vfs::OverlayFS,
    user_vfs: vfs::OverlayFS,
}

/// Represents a file, either in the filesystem, or in the resources zip file,
/// or whatever.
pub enum File {
    /// A wrapper for a VFile trait object.
    VfsFile(Box<dyn vfs::VFile>),
}

unsafe impl Send for File {}

impl fmt::Debug for File {
    // Make this more useful?
    // But we can't seem to get a filename out of a file,
    // soooooo.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            File::VfsFile(ref _file) => write!(f, "VfsFile"),
        }
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            File::VfsFile(ref mut f) => f.read(buf),
        }
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            File::VfsFile(ref mut f) => f.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            File::VfsFile(ref mut f) => f.flush(),
        }
    }
}

impl io::Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match *self {
            File::VfsFile(ref mut f) => f.seek(pos),
        }
    }
}

#[allow(unused)]
impl Filesystem {
    pub fn new() -> Filesystem {
        // Set up VFS to merge resource path, root path, and zip path.
        let overlay = vfs::OverlayFS::new();
        // User data VFS.
        let user_overlay = vfs::OverlayFS::new();

        Filesystem { vfs: overlay, user_vfs: user_overlay }
    }

    /// Opens the given `path` and returns the resulting `File`
    /// in read-only mode.
    pub(crate) fn open<P: AsRef<path::Path>>(&self, path: P) -> GameResult<File> {
        self.vfs.open(path.as_ref()).map(|f| File::VfsFile(f))
    }

    /// Opens the given `path` from user directory and returns the resulting `File`
    /// in read-only mode.
    pub(crate) fn user_open<P: AsRef<path::Path>>(&self, path: P) -> GameResult<File> {
        self.user_vfs.open(path.as_ref()).map(|f| File::VfsFile(f))
    }

    /// Opens a file in the user directory with the given
    /// [`filesystem::OpenOptions`](struct.OpenOptions.html).
    /// Note that even if you open a file read-write, it can only
    /// write to files in the "user" directory.
    pub(crate) fn open_options<P: AsRef<path::Path>>(&self, path: P, options: OpenOptions) -> GameResult<File> {
        self.user_vfs.open_options(path.as_ref(), options).map(|f| File::VfsFile(f)).map_err(|e| {
            GameError::ResourceLoadError(format!("Tried to open {:?} but got error: {:?}", path.as_ref(), e))
        })
    }

    /// Creates a new file in the user directory and opens it
    /// to be written to, truncating it if it already exists.
    pub(crate) fn user_create<P: AsRef<path::Path>>(&self, path: P) -> GameResult<File> {
        self.user_vfs.create(path.as_ref()).map(|f| File::VfsFile(f))
    }

    /// Create an empty directory in the user dir
    /// with the given name.  Any parents to that directory
    /// that do not exist will be created.
    pub(crate) fn user_create_dir<P: AsRef<path::Path>>(&self, path: P) -> GameResult<()> {
        self.user_vfs.mkdir(path.as_ref())
    }

    /// Deletes the specified file in the user dir.
    pub(crate) fn user_delete<P: AsRef<path::Path>>(&self, path: P) -> GameResult<()> {
        self.user_vfs.rm(path.as_ref())
    }

    /// Deletes the specified directory in the user dir,
    /// and all its contents!
    pub(crate) fn user_delete_dir<P: AsRef<path::Path>>(&self, path: P) -> GameResult<()> {
        self.user_vfs.rmrf(path.as_ref())
    }

    /// Check whether a file or directory in the user directory exists.
    pub(crate) fn user_exists<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.user_vfs.exists(path.as_ref())
    }

    /// Check whether a file or directory exists.
    pub(crate) fn exists<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.vfs.exists(path.as_ref())
    }

    /// Check whether a path points at a file.
    pub(crate) fn user_is_file<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.user_vfs.metadata(path.as_ref()).map(|m| m.is_file()).unwrap_or(false)
    }

    /// Check whether a path points at a file.
    pub(crate) fn is_file<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.vfs.metadata(path.as_ref()).map(|m| m.is_file()).unwrap_or(false)
    }

    /// Check whether a path points at a directory.
    pub(crate) fn user_is_dir<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.user_vfs.metadata(path.as_ref()).map(|m| m.is_dir()).unwrap_or(false)
    }

    /// Check whether a path points at a directory.
    pub(crate) fn is_dir<P: AsRef<path::Path>>(&self, path: P) -> bool {
        self.vfs.metadata(path.as_ref()).map(|m| m.is_dir()).unwrap_or(false)
    }

    /// Returns a list of all files and directories in the user directory,
    /// in no particular order.
    ///
    /// Lists the base directory if an empty path is given.
    pub(crate) fn user_read_dir<P: AsRef<path::Path>>(
        &self,
        path: P,
    ) -> GameResult<Box<dyn Iterator<Item = path::PathBuf>>> {
        let itr = self
            .user_vfs
            .read_dir(path.as_ref())?
            .map(|fname| fname.expect("Could not read file in read_dir()?  Should never happen, I hope!"));
        Ok(Box::new(itr))
    }

    /// Returns a list of all files and directories in the resource directory,
    /// in no particular order.
    ///
    /// Lists the base directory if an empty path is given.
    pub(crate) fn read_dir<P: AsRef<path::Path>>(
        &self,
        path: P,
    ) -> GameResult<Box<dyn Iterator<Item = path::PathBuf>>> {
        let itr = self
            .vfs
            .read_dir(path.as_ref())?
            .map(|fname| fname.expect("Could not read file in read_dir()?  Should never happen, I hope!"));
        Ok(Box::new(itr))
    }

    fn write_to_string(&self) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        for vfs in self.vfs.roots() {
            write!(s, "Source {:?}", vfs).expect("Could not write to string; should never happen?");
            match vfs.read_dir(path::Path::new("/")) {
                Ok(files) => {
                    for itm in files {
                        write!(s, "  {:?}", itm).expect("Could not write to string; should never happen?");
                    }
                }
                Err(e) => write!(s, " Could not read source: {:?}", e)
                    .expect("Could not write to string; should never happen?"),
            }
        }
        s
    }

    /// Adds the given (absolute) path to the list of directories
    /// it will search to look for resources.
    ///
    /// You probably shouldn't use this in the general case, since it is
    /// harder than it looks to make it bulletproof across platforms.
    /// But it can be very nice for debugging and dev purposes, such as
    /// by pushing `$CARGO_MANIFEST_DIR/resources` to it
    pub fn mount(&mut self, path: &path::Path, readonly: bool) {
        let physfs = vfs::PhysicalFS::new(path, readonly);
        trace!("Mounting new path: {:?}", physfs);
        self.vfs.push_back(Box::new(physfs));
    }

    pub fn mount_vfs(&mut self, vfs: Box<dyn vfs::VFS>) {
        self.vfs.push_back(vfs);
    }

    pub fn mount_user_vfs(&mut self, vfs: Box<dyn vfs::VFS>) {
        self.user_vfs.push_back(vfs);
    }

    pub fn unmount_vfs(&mut self, root: &PathBuf) {
        self.vfs.remove(root);
    }

    pub fn unmount_user_vfs(&mut self, root: &PathBuf) {
        self.user_vfs.remove(root);
    }
}

/// Opens the given path and returns the resulting `File`
/// in read-only mode.
pub fn open<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult<File> {
    ctx.filesystem.open(path)
}

pub fn open_find<P: AsRef<path::Path>>(ctx: &Context, roots: &Vec<String>, path: P) -> GameResult<File> {
    let mut errors = Vec::new();
    for root in roots {
        let mut full_path = root.to_string();
        full_path.push_str(path.as_ref().to_string_lossy().as_ref());

        let result = ctx.filesystem.open(&full_path);
        if result.is_ok() {
            return result;
        }

        errors.push((PathBuf::from(full_path), result.err().unwrap()));
    }

    Err(GameError::ResourceNotFound("File not found".to_owned(), errors))
}

/// Opens the given path in the user directory and returns the resulting `File`
/// in read-only mode.
pub fn user_open<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult<File> {
    ctx.filesystem.user_open(path)
}

/// Opens a file in the user directory with the given `filesystem::OpenOptions`.
pub fn open_options<P: AsRef<path::Path>>(ctx: &Context, path: P, options: OpenOptions) -> GameResult<File> {
    ctx.filesystem.open_options(path, options)
}

/// Creates a new file in the user directory and opens it
/// to be written to, truncating it if it already exists.
pub fn user_create<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult<File> {
    ctx.filesystem.user_create(path)
}

/// Create an empty directory in the user dir
/// with the given name.  Any parents to that directory
/// that do not exist will be created.
pub fn user_create_dir<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult {
    ctx.filesystem.user_create_dir(path.as_ref())
}

/// Deletes the specified file in the user dir.
pub fn user_delete<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult {
    ctx.filesystem.user_delete(path.as_ref())
}

/// Deletes the specified directory in the user dir,
/// and all its contents!
pub fn user_delete_dir<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult {
    ctx.filesystem.user_delete_dir(path.as_ref())
}

/// Check whether a file or directory exists.
pub fn user_exists<P: AsRef<path::Path>>(ctx: &Context, path: P) -> bool {
    ctx.filesystem.user_exists(path.as_ref())
}

/// Check whether a path points at a file.
pub fn user_is_file<P: AsRef<path::Path>>(ctx: &Context, path: P) -> bool {
    ctx.filesystem.user_is_file(path)
}

/// Check whether a path points at a directory.
pub fn user_is_dir<P: AsRef<path::Path>>(ctx: &Context, path: P) -> bool {
    ctx.filesystem.user_is_dir(path)
}

/// Returns a list of all files and directories in the user directory,
/// in no particular order.
///
/// Lists the base directory if an empty path is given.
pub fn user_read_dir<P: AsRef<path::Path>>(
    ctx: &Context,
    path: P,
) -> GameResult<Box<dyn Iterator<Item = path::PathBuf>>> {
    ctx.filesystem.user_read_dir(path)
}

/// Check whether a file or directory exists.
pub fn exists<P: AsRef<path::Path>>(ctx: &Context, path: P) -> bool {
    ctx.filesystem.exists(path.as_ref())
}

pub fn exists_find<P: AsRef<path::Path>>(ctx: &Context, roots: &Vec<String>, path: P) -> bool {
    for root in roots {
        let mut full_path = root.to_string();
        full_path.push_str(path.as_ref().to_string_lossy().as_ref());

        if ctx.filesystem.exists(full_path) {
            return true;
        }
    }

    false
}

/// Check whether a path points at a file.
pub fn is_file<P: AsRef<path::Path>>(ctx: &Context, path: P) -> bool {
    ctx.filesystem.is_file(path)
}

/// Check whether a path points at a directory.
pub fn is_dir<P: AsRef<path::Path>>(ctx: &Context, path: P) -> bool {
    ctx.filesystem.is_dir(path)
}

/// Returns a list of all files and directories in the resource directory,
/// in no particular order.
///
/// Lists the base directory if an empty path is given.
pub fn read_dir<P: AsRef<path::Path>>(ctx: &Context, path: P) -> GameResult<Box<dyn Iterator<Item = path::PathBuf>>> {
    ctx.filesystem.read_dir(path)
}

pub fn read_dir_find<P: AsRef<path::Path>>(
    ctx: &Context,
    roots: &Vec<String>,
    path: P,
) -> GameResult<Box<dyn Iterator<Item = path::PathBuf>>> {
    let mut files = Vec::new();

    for root in roots {
        let mut full_path = root.to_string();
        full_path.push_str(path.as_ref().to_string_lossy().as_ref());

        let result = ctx.filesystem.read_dir(full_path);
        if result.is_ok() {
            files.push(result.unwrap());
        }
    }

    Ok(Box::new(files.into_iter().flatten()))
}

/// Adds the given (absolute) path to the list of directories
/// it will search to look for resources.
///
/// You probably shouldn't use this in the general case, since it is
/// harder than it looks to make it bulletproof across platforms.
/// But it can be very nice for debugging and dev purposes, such as
/// by pushing `$CARGO_MANIFEST_DIR/resources` to it
pub fn mount(ctx: &mut Context, path: &path::Path, readonly: bool) {
    ctx.filesystem.mount(path, readonly)
}

/// Adds a VFS to the list of resource search locations.
pub fn mount_vfs(ctx: &mut Context, vfs: Box<dyn vfs::VFS>) {
    ctx.filesystem.mount_vfs(vfs)
}

/// Adds a VFS to the list of user data search locations.
pub fn mount_user_vfs(ctx: &mut Context, vfs: Box<dyn vfs::VFS>) {
    ctx.filesystem.mount_user_vfs(vfs)
}

/// Unmounts a VFS with a provided root path.
pub fn unmount_vfs(ctx: &mut Context, root: &PathBuf) {
    ctx.filesystem.unmount_vfs(root)
}

/// Unmounts a user VFS with a provided root path.
pub fn unmount_user_vfs(ctx: &mut Context, root: &PathBuf) {
    ctx.filesystem.unmount_user_vfs(root)
}
