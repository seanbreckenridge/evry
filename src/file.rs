//! File/tag related functionality
//!
//! This includes functions resolve where to,
//! write to, and read from tag files

use app_dirs::{self, AppDataType, AppInfo};
use std::{
    fs::{create_dir_all, read_to_string, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

/// static information about this application
///
/// Used to determine where to put local data on the users filesystem
const APP_INFO: AppInfo = AppInfo {
    name: "evry",
    author: "seanbreckenridge",
};

/// Keeps track of the user data dir, creates directories if they don't exist
#[derive(Debug, Default)]
pub struct LocalDir {
    pub root_dir: PathBuf,
}

impl LocalDir {
    /// Creates application/data directories if they don't exist
    pub fn new() -> Self {
        let dir_info: PathBuf = app_dirs::get_app_root(AppDataType::UserData, &APP_INFO)
            .expect("Couldn't get users local data directory");
        create_dir_all(dir_info.as_path()).expect("Could not create evry local directory");
        create_dir_all(dir_info.as_path().join(Path::new("data")))
            .expect("Could not create data directory");
        Self { root_dir: dir_info }
    }
}

/// read epoch time from a tag file
pub fn read_epoch_millis(filepath: &str) -> u128 {
    let millis_str = read_to_string(filepath).expect("Could not read tag information from file");
    millis_str
        .parse()
        .expect("Could not convert tag file information to integer")
}

/// A 'tag' is the name of some evry task
///
/// This is used to differentiate
/// different tasks/runs of evry from each other.
///
/// Holds metadata about the tag name,
/// and gives access to the underlying file.
///
/// ```
/// evry 2 months -sometool && run tool
/// evry 10 minutes -requestfile && wget ...
/// ```
#[derive(Debug)]
pub struct Tag {
    /// the name of this tag, like `requestfile`
    pub name: String,
    /// the computed location of this tag, like `~/.local/share/evry/data/requestfile`
    pub path: String,
}

impl Tag {
    /// Creates a new tag, resolves its `path`
    pub fn new(name: String, local_dir: &LocalDir) -> Self {
        let mut buf = local_dir.root_dir.clone();
        buf.push("data");
        buf.push(&name);
        let path = buf.into_os_string().into_string().unwrap();
        Self { name, path }
    }

    /// Returns whether or not the corresponding tag file exists
    pub fn file_exists(&self) -> bool {
        Path::new(&self.path).exists()
    }

    /// Reads from the tag file, returning when this tag was last run
    pub fn read_epoch_millis(&self) -> u128 {
        read_epoch_millis(&self.path)
    }

    /// Writes a number (epoch datetime) to this tagfile
    pub fn write(&self, time: u128) {
        let fp = File::create(&self.path).expect("Could not create tag file");
        let mut writer = BufWriter::new(&fp);
        write!(&mut writer, "{}", time).expect("Could not write to file")
    }
}
