use std::{fs, path::Path};

use crate::meta::Meta;

// Read, validate and sort directory entries.
pub fn read_directory_entries(path: &Path) -> Vec<Meta> {
  let entries = match fs::read_dir(path) {
    Ok(entries) => entries,
    Err(error) => {
      eprintln!("Failed to read directory {}: {}", path.display(), error);

      return Vec::new();
    }
  };

  let mut metadata_entries = Vec::new();

  for entry in entries {
    let entry = match entry {
      Ok(value) => value,
      Err(error) => {
        eprintln!("Failed to read directory entry: {error}");
        continue;
      }
    };

    match Meta::new(&entry.path()) {
      Ok(metadata) => metadata_entries.push(metadata),
      Err(error) => {
        eprintln!(
          "Failed to get metadata for {}: {}",
          entry.path().display(),
          error
        );
      }
    }
  }

  // Files first, directories last.
  // Sort paths within each group.
  metadata_entries.sort_by(|a, b| {
    a.kind
      .is_dir()
      .cmp(&b.kind.is_dir())
      .then_with(|| a.path.cmp(&b.path))
  });

  metadata_entries
}
