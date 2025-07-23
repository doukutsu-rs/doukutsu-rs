use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(not(any(target_os = "android", target_os = "horizon")))]
use rfd::FileDialog;

#[derive(Clone, Debug, Default)]
pub struct FilePickerParams {
    pub save: bool, /// Opens save file dialog
    pub multiple: bool,
    pub pick_dirs: bool,
    pub file_name: Option<String>, /// Starting file name.
    pub starting_dir: Option<PathBuf>,
    pub filters: HashMap<String, Vec<String>>, // filter name -> file extensions
}

impl FilePickerParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save(mut self, save: bool) -> Self {
        self.save = save;
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn pick_dirs(mut self, pick_dirs: bool) -> Self {
        self.pick_dirs = pick_dirs;
        self
    }

    pub fn file_name(mut self, file_name: Option<String>) -> Self {
        self.file_name = file_name;
        self
    }

    pub fn starting_dir(mut self, starting_dir: Option<PathBuf>) -> Self {
        self.starting_dir = starting_dir;
        self
    }

    pub fn filter(mut self, name: String, ext: Vec<String>) -> Self {
        self.filters.insert(name, ext);
        self
    }
}

#[cfg(not(any(target_os = "android", target_os = "horizon")))]
pub fn open_file_picker(params: &FilePickerParams) -> Option<Vec<PathBuf>> {
    log::trace!("Call a file picker dialog with params: {:?}", params.clone());
    let mut dialog = FileDialog::new();

    if let Some(filename) = params.file_name.clone() {
        dialog = dialog.set_file_name(filename);
    }

    if let Some(dir) = params.starting_dir.clone() {
        dialog = dialog.set_directory(dir);
    }

    for filter in params.filters.iter() {
        dialog = dialog.add_filter(filter.0, filter.1);
    }

    let selected_files = match (params.pick_dirs, params.multiple, params.save) {
        (_, _, true) => dialog.save_file().map(|path| vec![path]),
        (true, false, _) => dialog.pick_folder().map(|path| vec![path]),
        (true, true, _) => dialog.pick_folders(),
        (false, false, _) => dialog.pick_file().map(|path| vec![path]),
        (false, true, _) => dialog.pick_files(),
    };

    log::trace!("Selected file entries: {:?}", selected_files.clone());

    return selected_files;
}

#[cfg(any(target_os = "android", target_os = "horizon"))]
pub fn open_file_picker(params: &FileChooserParams) -> Option<Vec<PathBuf>> {
    None
}