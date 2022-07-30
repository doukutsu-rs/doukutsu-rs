use std::fmt::Debug;
use std::io::Cursor;
use std::io::ErrorKind;
use std::io::SeekFrom;
use std::path::{Component, Path, PathBuf};
use std::{fmt, io};

use crate::framework::error::GameError::FilesystemError;
use crate::framework::error::GameResult;
use crate::framework::vfs::{OpenOptions, VFile, VMetadata, VFS};

#[derive(Debug)]
pub struct BuiltinFile(Cursor<&'static [u8]>);

impl BuiltinFile {
    pub fn from(buf: &'static [u8]) -> Box<dyn VFile> {
        Box::new(BuiltinFile(Cursor::new(buf)))
    }
}

impl io::Read for BuiltinFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl io::Seek for BuiltinFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}

impl io::Write for BuiltinFile {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(ErrorKind::PermissionDenied, "Built-in file system is read-only."))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(ErrorKind::PermissionDenied, "Built-in file system is read-only."))
    }
}

struct BuiltinMetadata {
    is_dir: bool,
    size: u64,
}

impl VMetadata for BuiltinMetadata {
    fn is_dir(&self) -> bool {
        self.is_dir
    }

    fn is_file(&self) -> bool {
        !self.is_dir
    }

    fn len(&self) -> u64 {
        self.size
    }
}

#[derive(Clone, Debug)]
enum FSNode {
    File(&'static str, &'static [u8]),
    Directory(&'static str, Vec<FSNode>),
}

impl FSNode {
    fn get_name(&self) -> &'static str {
        match self {
            FSNode::File(name, _) => name,
            FSNode::Directory(name, _) => name,
        }
    }

    fn to_file(&self) -> GameResult<Box<dyn VFile>> {
        match self {
            FSNode::File(_, buf) => Ok(BuiltinFile::from(buf)),
            FSNode::Directory(name, _) => Err(FilesystemError(format!("{} is a directory.", name))),
        }
    }

    fn to_metadata(&self) -> Box<dyn VMetadata> {
        match self {
            FSNode::File(_, buf) => Box::new(BuiltinMetadata { is_dir: false, size: buf.len() as u64 }),
            FSNode::Directory(_, _) => Box::new(BuiltinMetadata { is_dir: true, size: 0 }),
        }
    }
}

pub struct BuiltinFS {
    root: Vec<FSNode>,
}

impl BuiltinFS {
    pub fn new() -> Self {
        Self {
            root: vec![FSNode::Directory(
                "builtin",
                vec![
                    FSNode::File("builtin_font.fnt", include_bytes!("builtin/builtin_font.fnt")),
                    FSNode::File("builtin_font_0.png", include_bytes!("builtin/builtin_font_0.png")),
                    FSNode::File("builtin_font_1.png", include_bytes!("builtin/builtin_font_1.png")),
                    FSNode::File(
                        "organya-wavetable-doukutsu.bin",
                        include_bytes!("builtin/organya-wavetable-doukutsu.bin"),
                    ),
                    FSNode::File("touch.png", include_bytes!("builtin/touch.png")),
                    FSNode::Directory(
                        "builtin_data",
                        vec![
                            FSNode::File("buttons.png", include_bytes!("builtin/builtin_data/buttons.png")),
                            FSNode::File("triangles.png", include_bytes!("builtin/builtin_data/triangles.png")),
                        ],
                    ),
                    FSNode::Directory(
                        "shaders",
                        vec![
                            // FSNode::File("basic_150.vert.glsl", include_bytes!("builtin/shaders/basic_150.vert.glsl")),
                            // FSNode::File("water_150.frag.glsl", include_bytes!("builtin/shaders/water_150.frag.glsl")),
                            // FSNode::File("basic_es300.vert.glsl", include_bytes!("builtin/shaders/basic_es300.vert.glsl")),
                            // FSNode::File("water_es300.frag.glsl", include_bytes!("builtin/shaders/water_es300.frag.glsl")),
                        ],
                    ),
                    FSNode::Directory(
                        "lightmap",
                        vec![FSNode::File("spot.png", include_bytes!("builtin/lightmap/spot.png"))],
                    ),
                    FSNode::Directory(
                        "locale",
                        vec![
                            FSNode::File("en.json", include_bytes!("builtin/locale/en.json")),
                            FSNode::File("jp.json", include_bytes!("builtin/locale/jp.json")),
                        ],
                    ),
                ],
            )],
        }
    }

    fn get_node(&self, path: &Path) -> GameResult<FSNode> {
        let mut iter = path.components().peekable();

        if let Some(Component::RootDir) = iter.next() {
            let mut curr_dir = &self.root;

            if iter.peek().is_none() {
                return Ok(FSNode::Directory("", self.root.clone()));
            }

            while let Some(comp) = iter.next() {
                let comp_name = comp.as_os_str().to_string_lossy();

                for file in curr_dir {
                    match file {
                        FSNode::File(name, _) if comp_name.eq(name) => {
                            return if iter.peek().is_some() {
                                Err(FilesystemError(format!("Expected a directory, found a file: {:?}", path)))
                            } else {
                                Ok(file.clone())
                            };
                        }
                        FSNode::Directory(name, contents) if comp_name.eq(name) => {
                            if iter.peek().is_some() {
                                curr_dir = contents;
                                break;
                            } else {
                                return Ok(file.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        } else {
            return Err(FilesystemError("Path must be absolute.".to_string()));
        }

        Err(FilesystemError("File not found.".to_string()))
    }
}

impl Debug for BuiltinFS {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<BuiltinFS>")
    }
}

impl VFS for BuiltinFS {
    fn open_options(&self, path: &Path, open_options: OpenOptions) -> GameResult<Box<dyn VFile>> {
        if open_options.write || open_options.create || open_options.append || open_options.truncate {
            let msg = format!("Cannot alter file {:?} in root {:?}, filesystem read-only", path, self);
            return Err(FilesystemError(msg));
        }

        self.get_node(path)?.to_file()
    }

    fn mkdir(&self, _path: &Path) -> GameResult<()> {
        Err(FilesystemError("Tried to make directory {} but FS is read-only".to_string()))
    }

    fn rm(&self, _path: &Path) -> GameResult<()> {
        Err(FilesystemError("Tried to remove file {} but FS is read-only".to_string()))
    }

    fn rmrf(&self, _path: &Path) -> GameResult<()> {
        Err(FilesystemError("Tried to remove file/dir {} but FS is read-only".to_string()))
    }

    fn exists(&self, path: &Path) -> bool {
        self.get_node(path).is_ok()
    }

    fn metadata(&self, path: &Path) -> GameResult<Box<dyn VMetadata>> {
        self.get_node(path).map(|v| v.to_metadata())
    }

    fn read_dir(&self, path: &Path) -> GameResult<Box<dyn Iterator<Item = GameResult<PathBuf>>>> {
        match self.get_node(path) {
            Ok(FSNode::Directory(_, contents)) => {
                let mut vec = Vec::new();
                for node in contents {
                    vec.push(Ok(PathBuf::from(node.get_name())))
                }

                Ok(Box::new(vec.into_iter()))
            }
            Ok(FSNode::File(_, _)) => Err(FilesystemError(format!("Expected a directory, found a file: {:?}", path))),
            Err(e) => Err(e),
        }
    }

    fn to_path_buf(&self) -> Option<PathBuf> {
        None
    }
}

#[test]
fn test_builtin_fs() {
    let fs = BuiltinFS {
        root: vec![
            FSNode::File("test.txt", &[]),
            FSNode::Directory(
                "memes",
                vec![
                    FSNode::File("nothing.txt", &[]),
                    FSNode::Directory("secret stuff", vec![FSNode::File("passwords.txt", b"12345678")]),
                ],
            ),
            FSNode::File("test2.txt", &[]),
        ],
    };

    println!("{:?}", fs.get_node(Path::new("/")).unwrap());
    println!("{:?}", fs.get_node(Path::new("/test.txt")).unwrap());
    println!("{:?}", fs.get_node(Path::new("/memes")).unwrap());
    println!("{:?}", fs.get_node(Path::new("/memes/nothing.txt")).unwrap());
    println!("{:?}", fs.get_node(Path::new("/memes/secret stuff/passwords.txt")).unwrap());
}
