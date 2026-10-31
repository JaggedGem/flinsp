use std::{fs, io, path::Path};

use crate::{
  hash::{hash_file, hash_folder},
  meta::Meta,
};

pub struct Entry {
  pub meta: Meta,
  pub hash: Option<String>,
  pub children: Option<Vec<Entry>>,
}

pub(crate) fn build_tree(
  path: &Path,
  include_hashes: bool,
  is_recursive: bool,
) -> io::Result<Entry> {
  let meta = Meta::new(path)?;

  if meta.kind.is_file() {
    return Ok(Entry {
      meta,
      hash: if include_hashes {
        hash_file(path).ok()
      } else {
        None
      },
      children: None,
    });
  }

  let entries = read_directory_entries(path);

  let mut parent_children: Vec<Entry> = Vec::new();

  // Populate the children entries
  for child in entries {
    let child_node = build_tree(&child.path, include_hashes, is_recursive)?;

    parent_children.push(child_node);
  }

  // Calculate parent's hash if needed
  let parent_hash: Option<String> = if include_hashes {
    let mut children_hashes: Vec<(&Path, String)> = Vec::with_capacity(parent_children.len());

    for child in &parent_children {
      let hash = match &child.hash {
        Some(h) => h.clone(),
        None => {
          eprintln!(
            "Failed to generate parent directory's hash because one of the child's hash was empty: {}",
            child.meta.path.display()
          );

          break;
        }
      };

      children_hashes.push((&child.meta.path, hash));
    }

    if children_hashes.len() == parent_children.len() {
      match hash_folder(&children_hashes) {
        Ok(h) => Some(h),
        Err(e) => {
          eprintln!(
            "Failed to generate parent directory's hash for {}: {e}",
            meta.path.display()
          );
          None
        }
      }
    } else {
      None
    }
  } else {
    None
  };

  let parent = Entry {
    meta,
    hash: parent_hash,
    children: if is_recursive {
      Some(parent_children)
    } else {
      None
    },
  };

  Ok(parent)
}

// Read, validate and sort directory entries.
fn read_directory_entries(path: &Path) -> Vec<Meta> {
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
