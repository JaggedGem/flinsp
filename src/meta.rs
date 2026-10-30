use std::{
  ffi::OsString,
  fs::{self, FileType},
  io::Error,
  path::{Path, PathBuf, absolute},
  time::SystemTime,
};

use crate::hash::{hash_file, hash_folder};

pub struct Meta {
  pub name: OsString,
  pub extension: String,
  pub size: u64,
  pub last_modified: Option<SystemTime>,
  pub last_accessed: Option<SystemTime>,
  pub created_at: Option<SystemTime>,
  pub kind: FileType,
  pub path: PathBuf,
  pub hash: Option<String>,
}

impl Meta {
  pub fn new(path: &Path, include_hashes: bool) -> Result<Self, Error> {
    let metadata = fs::metadata(path)?;

    let extension = &path
      .extension()
      .and_then(|ext| ext.to_str())
      .unwrap_or("")
      .to_string();

    let absolute_path = absolute(path).unwrap_or_else(|_| path.to_owned());

    Ok(Self {
      name: match path.file_stem() {
        Some(val) => val.to_owned(),
        None => OsString::from(""),
      },
      extension: String::from(extension),
      size: metadata.len(),
      last_modified: metadata.modified().ok(),
      last_accessed: metadata.accessed().ok(),
      created_at: metadata.created().ok(),
      kind: metadata.file_type(),
      path: absolute_path,
      hash: if include_hashes {
        if metadata.is_file() {
          hash_file(path).ok()
        } else if metadata.is_dir() {
          hash_folder(path).ok()
        } else {
          None
        }
      } else {
        None
      },
    })
  }

  pub fn human_readable_size(&self) -> String {
    const KB: u64 = 1000;
    const MB: u64 = KB * 1000;
    const GB: u64 = MB * 1000;
    const TB: u64 = GB * 1000;

    match self.size {
      0..KB => format!("{} Bytes", self.size),
      KB..MB => format!("{:.2} Kilobytes", self.size as f64 / KB as f64),
      MB..GB => format!("{:.2} Megabytes", self.size as f64 / MB as f64),
      GB..TB => format!("{:.2} Gigabytes", self.size as f64 / GB as f64),
      _ => format!("{:.2} Terabytes", self.size as f64 / TB as f64),
    }
  }
}
