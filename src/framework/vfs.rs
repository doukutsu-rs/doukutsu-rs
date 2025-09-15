//! A virtual file system layer that lets us define multiple
//! "file systems" with various backing stores, then merge them
//! together.
//!
//! Basically a re-implementation of the C library `PhysFS`.  The
//! `vfs` crate does something similar but has a couple design
//! decisions that make it kind of incompatible with this use case:
//! the relevant trait for it has generic methods so we can't use it
//! as a trait object, and its path abstraction is not the most
//! convenient.

use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fmt::{self, Debug};
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{self, Component, Path, PathBuf};

use crate::framework::error::{GameError, GameResult};

fn convenient_path_to_str(path: &path::Path) -> GameResult<&str> {
    path.to_str().ok_or_else(|| {
        let errmessage = format!("Invalid path format for resource: {:?}", path);
        GameError::FilesystemError(errmessage)
    })
}

/// Virtual file
pub trait VFile: Read + Write + Seek + Debug + Send + Sync {}

impl<T> VFile for T where T: Read + Write + Seek + Debug + Send + Sync {}

/// Options for opening files
///
/// We need our own version of this structure because the one in
/// `std` annoyingly doesn't let you read the read/write/create/etc
/// state out of it.
#[must_use]
#[allow(missing_docs)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct OpenOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub append: bool,
    pub truncate: bool,
}

#[allow(unused)]
impl OpenOptions {
    /// Create a new instance
    pub fn new() -> OpenOptions {
        Default::default()
    }

    /// Open for reading
    pub fn read(mut self, read: bool) -> OpenOptions {
        self.read = read;
        self
    }

    /// Open for writing
    pub fn write(mut self, write: bool) -> OpenOptions {
        self.write = write;
        self
    }

    /// Create the file if it does not exist yet
    pub fn create(mut self, create: bool) -> OpenOptions {
        self.create = create;
        self
    }

    /// Append at the end of the file
    pub fn append(mut self, append: bool) -> OpenOptions {
        self.append = append;
        self
    }

    /// Truncate the file to 0 bytes after opening
    pub fn truncate(mut self, truncate: bool) -> OpenOptions {
        self.truncate = truncate;
        self
    }

    fn to_fs_openoptions(self) -> fs::OpenOptions {
        let mut opt = fs::OpenOptions::new();
        let _ = opt
            .read(self.read)
            .write(self.write)
            .create(self.create)
            .append(self.append)
            .truncate(self.truncate)
            .create(self.create);
        opt
    }
}

/// Virtual filesystem
pub trait VFS: Debug {
    /// Open the file at this path with the given options
    fn open_options(&self, path: &Path, open_options: OpenOptions) -> GameResult<Box<dyn VFile>>;
    /// Open the file at this path for reading
    fn open(&self, path: &Path) -> GameResult<Box<dyn VFile>> {
        self.open_options(path, OpenOptions::new().read(true))
    }
    /// Open the file at this path for writing, truncating it if it exists already
    fn create(&self, path: &Path) -> GameResult<Box<dyn VFile>> {
        self.open_options(path, OpenOptions::new().write(true).create(true).truncate(true))
    }
    /// Open the file at this path for appending, creating it if necessary
    fn append(&self, path: &Path) -> GameResult<Box<dyn VFile>> {
        self.open_options(path, OpenOptions::new().write(true).create(true).append(true))
    }
    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> GameResult;

    /// Remove a file or an empty directory.
    fn rm(&self, path: &Path) -> GameResult;

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> GameResult;

    /// Check if the file exists
    fn exists(&self, path: &Path) -> bool;

    /// Get the file's metadata
    fn metadata(&self, path: &Path) -> GameResult<Box<dyn VMetadata>>;

    /// Retrieve all file and directory entries in the given directory.
    fn read_dir(&self, path: &Path) -> GameResult<Box<dyn Iterator<Item = GameResult<PathBuf>>>>;

    /// Retrieve the actual location of the VFS root, if available.
    fn to_path_buf(&self) -> Option<PathBuf>;
}

/// VFS metadata
pub trait VMetadata {
    /// Returns whether or not it is a directory.
    /// Note that zip files don't actually have directories, awkwardly,
    /// just files with very long names.
    fn is_dir(&self) -> bool;
    /// Returns whether or not it is a file.
    fn is_file(&self) -> bool;
    /// Returns the length of the thing.  If it is a directory,
    /// the result of this is undefined/platform dependent.
    fn len(&self) -> u64;
}

/// A VFS that points to a directory and uses it as the root of its
/// file hierarchy.
///
/// It IS allowed to have symlinks in it!  They're surprisingly
/// difficult to get rid of.
#[derive(Clone)]
pub struct PhysicalFS {
    root: PathBuf,
    readonly: bool,
    lowercase: bool,
}

#[derive(Debug, Clone)]
/// Physical FS metadata
pub struct PhysicalMetadata(fs::Metadata);

impl VMetadata for PhysicalMetadata {
    fn is_dir(&self) -> bool {
        self.0.is_dir()
    }
    fn is_file(&self) -> bool {
        self.0.is_file()
    }
    fn len(&self) -> u64 {
        self.0.len()
    }
}

/// This takes an absolute path and returns either a sanitized relative
/// version of it, or None if there's something bad in it.
///
/// What we want is an absolute path with no `..`'s in it, so, something
/// like "/foo" or "/foo/bar.txt".  This means a path with components
/// starting with a `RootDir`, and zero or more `Normal` components.
///
/// We gotta return a new path because there's apparently no real good way
/// to turn an absolute path into a relative path with the same
/// components (other than the first), and pushing an absolute `Path`
/// onto a `PathBuf` just completely nukes its existing contents.
fn sanitize_path(path: &path::Path) -> Option<PathBuf> {
    let mut c = path.components();
    match c.next() {
        Some(path::Component::RootDir) => (),
        _ => return None,
    }

    fn is_normal_component(comp: path::Component) -> Option<&str> {
        match comp {
            path::Component::Normal(s) => s.to_str(),
            _ => None,
        }
    }

    // This could be done more cleverly but meh
    let mut accm = PathBuf::new();
    for component in c {
        if let Some(s) = is_normal_component(component) {
            accm.push(s)
        } else {
            return None;
        }
    }
    Some(accm)
}

impl PhysicalFS {
    /// Creates a new PhysicalFS
    pub fn new(root: &Path, readonly: bool) -> Self {
        PhysicalFS { root: root.into(), readonly, lowercase: false }
    }

    pub fn new_lowercase(root: &Path) -> Self {
        PhysicalFS { root: root.into(), readonly: true, lowercase: true }
    }

    /// Takes a given path (&str) and returns
    /// a new PathBuf containing the canonical
    /// absolute path you get when appending it
    /// to this filesystem's root.
    fn to_absolute(&self, p: &Path) -> GameResult<PathBuf> {
        if let Some(mut safe_path) = sanitize_path(p) {
            if self.lowercase {
                safe_path = PathBuf::from(p.to_string_lossy().to_lowercase())
            }

            let mut root_path = self.root.clone();
            root_path.push(safe_path.clone());

            // emulate case insensitive paths on systems with case sensitive filesystems.
            #[cfg(not(any(target_os = "windows", target_os = "macos")))]
            if !root_path.exists() {
                let mut root_path2 = self.root.clone();
                let mut ok = true;

                let components: Vec<&OsStr> = safe_path
                    .components()
                    .filter_map(|c| if let Component::Normal(s) = c { Some(s) } else { None })
                    .collect();

                'citer: for node in components {
                    let mut tmp = root_path2.clone();
                    tmp.push(node);
                    if tmp.exists() {
                        root_path2 = tmp;
                        continue;
                    }

                    let node_lower = node.to_ascii_lowercase();
                    if let Ok(entries) = root_path2.read_dir() {
                        for entry in entries.flatten() {
                            let name = entry.file_name();
                            if name.to_ascii_lowercase() != node_lower {
                                continue;
                            }

                            root_path2.push(name);
                            continue 'citer;
                        }
                    }

                    ok = false;
                    break;
                }

                if ok {
                    // log::info!("resolved case insensitive path {:?} -> {:?}", root_path, root_path2);
                    root_path = root_path2;
                }
            }

            Ok(root_path)
        } else {
            let msg = format!(
                "Path {:?} is not valid: must be an absolute path with no \
                 references to parent directories",
                p
            );
            Err(GameError::FilesystemError(msg))
        }
    }

    /// Creates the PhysicalFS's root directory if necessary.
    /// Idempotent.
    /// This way we can not create the directory until it's
    /// actually used, though it IS a tiny bit of a performance
    /// malus.
    fn create_root(&self) -> GameResult {
        if !self.root.exists() {
            fs::create_dir_all(&self.root).map_err(GameError::from)
        } else {
            Ok(())
        }
    }
}

impl Debug for PhysicalFS {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<PhysicalFS root: {}>", self.root.display())
    }
}

impl VFS for PhysicalFS {
    /// Open the file at this path with the given options
    fn open_options(&self, path: &Path, open_options: OpenOptions) -> GameResult<Box<dyn VFile>> {
        if self.readonly && (open_options.write || open_options.create || open_options.append || open_options.truncate)
        {
            let msg = format!("Cannot alter file {:?} in root {:?}, filesystem read-only", path, self);
            return Err(GameError::FilesystemError(msg));
        }
        self.create_root()?;
        let p = self.to_absolute(path)?;
        open_options.to_fs_openoptions().open(p).map(|x| Box::new(x) as Box<dyn VFile>).map_err(GameError::from)
    }

    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> GameResult {
        if self.readonly {
            return Err(GameError::FilesystemError(
                "Tried to make directory {} but FS is \
                 read-only"
                    .to_string(),
            ));
        }
        self.create_root()?;
        let p = self.to_absolute(path)?;
        //println!("Creating {:?}", p);
        fs::DirBuilder::new().recursive(true).create(p).map_err(GameError::from)
    }

    /// Remove a file
    fn rm(&self, path: &Path) -> GameResult {
        if self.readonly {
            return Err(GameError::FilesystemError("Tried to remove file {} but FS is read-only".to_string()));
        }

        self.create_root()?;
        let p = self.to_absolute(path)?;
        if p.is_dir() {
            fs::remove_dir(p).map_err(GameError::from)
        } else {
            fs::remove_file(p).map_err(GameError::from)
        }
    }

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> GameResult {
        if self.readonly {
            return Err(GameError::FilesystemError(
                "Tried to remove file/dir {} but FS is \
                 read-only"
                    .to_string(),
            ));
        }

        self.create_root()?;
        let p = self.to_absolute(path)?;
        if p.is_dir() {
            fs::remove_dir_all(p).map_err(GameError::from)
        } else {
            fs::remove_file(p).map_err(GameError::from)
        }
    }

    /// Check if the file exists
    fn exists(&self, path: &Path) -> bool {
        match self.to_absolute(path) {
            Ok(p) => p.exists(),
            _ => false,
        }
    }

    /// Get the file's metadata
    fn metadata(&self, path: &Path) -> GameResult<Box<dyn VMetadata>> {
        self.create_root()?;
        let p = self.to_absolute(path)?;
        p.metadata().map(|m| Box::new(PhysicalMetadata(m)) as Box<dyn VMetadata>).map_err(GameError::from)
    }

    /// Retrieve the path entries in this path
    fn read_dir(&self, path: &Path) -> GameResult<Box<dyn Iterator<Item = GameResult<PathBuf>>>> {
        self.create_root()?;
        let p = self.to_absolute(path)?;
        // This is inconvenient because path() returns the full absolute
        // path of the bloody file, which is NOT what we want!
        // But if we use file_name() to just get the name then it is ALSO not what we want!
        // what we WANT is the full absolute file path, *relative to the resources dir*.
        // So that we can do read_dir("/foobar/"), and for each file, open it and query
        // it and such by name.
        // So we build the paths ourself.
        let direntry_to_path = |entry: &fs::DirEntry| -> GameResult<PathBuf> {
            let fname =
                entry.file_name().into_string().expect("Non-unicode char in file path?  Should never happen, I hope!");
            let mut pathbuf = PathBuf::from(path);
            pathbuf.push(fname);
            Ok(pathbuf)
        };
        let itr = fs::read_dir(p)?.map(|entry| direntry_to_path(&entry?)).collect::<Vec<_>>().into_iter();
        Ok(Box::new(itr))
    }

    /// Retrieve the actual location of the VFS root, if available.
    fn to_path_buf(&self) -> Option<PathBuf> {
        Some(self.root.clone())
    }
}

/// A structure that joins several VFS's together in order.
#[derive(Debug)]
pub struct OverlayFS {
    roots: VecDeque<Box<dyn VFS>>,
}

impl OverlayFS {
    /// Creates a new OverlayFS
    pub fn new() -> Self {
        Self { roots: VecDeque::new() }
    }

    /// Adds a new VFS to the front of the list.
    /// Currently unused, I suppose, but good to
    /// have at least for tests.
    #[allow(dead_code)]
    pub fn push_front(&mut self, fs: Box<dyn VFS>) {
        self.roots.push_front(fs);
    }

    /// Adds a new VFS to the end of the list.
    pub fn push_back(&mut self, fs: Box<dyn VFS>) {
        self.roots.push_back(fs);
    }

    /// Returns a list of registered VFS roots.
    pub fn roots(&self) -> &VecDeque<Box<dyn VFS>> {
        &self.roots
    }

    /// Removes a VFS with a provided root.
    pub fn remove(&mut self, root: &PathBuf) {
        self.roots.iter().position(|fs| fs.to_path_buf() == Some(root.clone())).map(|i| self.roots.remove(i));
    }
}

impl VFS for OverlayFS {
    /// Open the file at this path with the given options
    fn open_options(&self, path: &Path, open_options: OpenOptions) -> GameResult<Box<dyn VFile>> {
        let mut tried: Vec<(PathBuf, GameError)> = vec![];

        for vfs in &self.roots {
            match vfs.open_options(path, open_options) {
                Err(e) => {
                    if let Some(vfs_path) = vfs.to_path_buf() {
                        tried.push((vfs_path, e));
                    } else {
                        tried.push((PathBuf::from("<invalid path>"), e));
                    }
                }
                f => return f,
            }
        }
        let errmessage = String::from(convenient_path_to_str(path)?);
        Err(GameError::ResourceNotFound(errmessage, tried))
    }

    /// Create a directory at the location by this path
    fn mkdir(&self, path: &Path) -> GameResult {
        for vfs in &self.roots {
            match vfs.mkdir(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(GameError::FilesystemError(format!("Could not find anywhere writeable to make dir {:?}", path)))
    }

    /// Remove a file
    fn rm(&self, path: &Path) -> GameResult {
        for vfs in &self.roots {
            match vfs.rm(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(GameError::FilesystemError(format!("Could not remove file {:?}", path)))
    }

    /// Remove a file or directory and all its contents
    fn rmrf(&self, path: &Path) -> GameResult {
        for vfs in &self.roots {
            match vfs.rmrf(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(GameError::FilesystemError(format!("Could not remove file/dir {:?}", path)))
    }

    /// Check if the file exists
    fn exists(&self, path: &Path) -> bool {
        for vfs in &self.roots {
            if vfs.exists(path) {
                return true;
            }
        }

        false
    }

    /// Get the file's metadata
    fn metadata(&self, path: &Path) -> GameResult<Box<dyn VMetadata>> {
        for vfs in &self.roots {
            match vfs.metadata(path) {
                Err(_) => (),
                f => return f,
            }
        }
        Err(GameError::FilesystemError(format!("Could not get metadata for file/dir {:?}", path)))
    }

    /// Retrieve the path entries in this path
    fn read_dir(&self, path: &Path) -> GameResult<Box<dyn Iterator<Item = GameResult<PathBuf>>>> {
        // This is tricky 'cause we have to actually merge iterators together...
        // Doing it the simple and stupid way works though.
        let mut v = Vec::new();
        for fs in &self.roots {
            if let Ok(rddir) = fs.read_dir(path) {
                v.extend(rddir)
            }
        }
        Ok(Box::new(v.into_iter()))
    }

    /// Retrieve the actual location of the VFS root, if available.
    fn to_path_buf(&self) -> Option<PathBuf> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufRead};

    use super::*;

    #[test]
    fn headless_test_path_filtering() {
        // Valid pahts
        let p = path::Path::new("/foo");
        assert!(sanitize_path(p).is_some());

        let p = path::Path::new("/foo/");
        assert!(sanitize_path(p).is_some());

        let p = path::Path::new("/foo/bar.txt");
        assert!(sanitize_path(p).is_some());

        let p = path::Path::new("/");
        assert!(sanitize_path(p).is_some());

        // Invalid paths
        let p = path::Path::new("../foo");
        assert!(sanitize_path(p).is_none());

        let p = path::Path::new("foo");
        assert!(sanitize_path(p).is_none());

        let p = path::Path::new("/foo/../../");
        assert!(sanitize_path(p).is_none());

        let p = path::Path::new("/foo/../bop");
        assert!(sanitize_path(p).is_none());

        let p = path::Path::new("/../bar");
        assert!(sanitize_path(p).is_none());

        let p = path::Path::new("");
        assert!(sanitize_path(p).is_none());
    }

    #[test]
    fn headless_test_read() {
        let cargo_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fs = PhysicalFS::new(cargo_path, true);
        let f = fs.open(Path::new("/Cargo.toml")).unwrap();
        let mut bf = io::BufReader::new(f);
        let mut s = String::new();
        let _ = bf.read_line(&mut s).unwrap();
        // Trim whitespace from string 'cause it will
        // potentially be different on Windows and Unix.
        let trimmed_string = s.trim();
        assert_eq!(trimmed_string, "[package]");
    }

    #[test]
    fn headless_test_read_overlay() {
        let cargo_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fs1 = PhysicalFS::new(cargo_path, true);
        let mut f2path = PathBuf::from(cargo_path);
        f2path.push("src");
        let fs2 = PhysicalFS::new(&f2path, true);
        let mut ofs = OverlayFS::new();
        ofs.push_back(Box::new(fs1));
        ofs.push_back(Box::new(fs2));

        assert!(ofs.exists(Path::new("/Cargo.toml")));
        assert!(ofs.exists(Path::new("/lib.rs")));
        assert!(!ofs.exists(Path::new("/foobaz.rs")));
    }

    #[test]
    fn headless_test_physical_all() {
        let cargo_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fs = PhysicalFS::new(cargo_path, false);
        let testdir = Path::new("/testdir");
        let f1 = Path::new("/testdir/file1.txt");

        // Delete testdir if it is still lying around
        if fs.exists(testdir) {
            fs.rmrf(testdir).unwrap();
        }
        assert!(!fs.exists(testdir));

        // Create and delete test dir
        fs.mkdir(testdir).unwrap();
        assert!(fs.exists(testdir));
        fs.rm(testdir).unwrap();
        assert!(!fs.exists(testdir));

        let test_string = "Foo!";
        fs.mkdir(testdir).unwrap();
        {
            let mut f = fs.append(f1).unwrap();
            let _ = f.write(test_string.as_bytes()).unwrap();
        }
        {
            let mut buf = Vec::new();
            let mut f = fs.open(f1).unwrap();
            let _ = f.read_to_end(&mut buf).unwrap();
            assert_eq!(&buf[..], test_string.as_bytes());
        }

        {
            // Test metadata()
            let m = fs.metadata(f1).unwrap();
            assert!(m.is_file());
            assert!(!m.is_dir());
            assert_eq!(m.len(), 4);

            let m = fs.metadata(testdir).unwrap();
            assert!(!m.is_file());
            assert!(m.is_dir());
            // Not exactly sure what the "length" of a directory is, buuuuuut...
            // It appears to vary based on the platform in fact.
            // On my desktop, it's 18.
            // On Travis's VM, it's 4096.
            // On Appveyor's VM, it's 0.
            // So, it's meaningless.
            //assert_eq!(m.len(), 18);
        }

        {
            // Test read_dir()
            let r = fs.read_dir(testdir).unwrap();
            assert_eq!(r.count(), 1);
            let r = fs.read_dir(testdir).unwrap();
            for f in r {
                let fname = f.unwrap();
                assert!(fs.exists(&fname));
            }
        }

        {
            assert!(fs.exists(f1));
            fs.rm(f1).unwrap();
            assert!(!fs.exists(f1));
        }

        fs.rmrf(testdir).unwrap();
        assert!(!fs.exists(testdir));
    }

    // BUGGO: TODO: Make sure all functions are tested for OverlayFS and ZipFS!!
}
