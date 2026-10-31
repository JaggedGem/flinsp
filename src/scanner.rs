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

fn calculate_parent_hash(children: &[Entry]) -> Option<String> {
  let mut children_hashes: Vec<(&Path, String)> = Vec::with_capacity(children.len());

  for child in children {
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

  if children_hashes.len() == children.len() {
    let hashes: Vec<(&Path, &str)> = children_hashes
      .iter()
      .map(|(p, h)| (*p, h.as_str()))
      .collect();

    match hash_folder(&hashes) {
      Ok(h) => Some(h),
      Err(e) => {
        eprintln!("Failed to generate parent directory's hash: {e}",);
        None
      }
    }
  } else {
    None
  }
}

pub(crate) fn build_tree(
  path: &Path,
  include_hashes: bool,
  is_recursive: bool,
  follow_symlinks: bool,
) -> io::Result<Entry> {
  let meta = Meta::new(path, follow_symlinks)?;

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

  let entries = read_directory_entries(path, follow_symlinks);

  let parent_children: Option<Vec<Entry>> = match include_hashes || is_recursive {
    true => {
      let mut parent_children: Vec<Entry> = Vec::new();

      // Populate the children entries
      for child in entries {
        let child_node = build_tree(&child.path, include_hashes, is_recursive, follow_symlinks)?;

        parent_children.push(child_node);
      }

      Some(parent_children)
    }
    false => None,
  };

  // Calculate parent's hash if needed
  let parent_hash: Option<String> = if include_hashes {
    if let Some(children) = &parent_children {
      calculate_parent_hash(children)
    } else {
      eprintln!(
        "Failed to generate parent directory's hash because it has no children: {}",
        path.display()
      );

      None
    }
  } else {
    None
  };

  let parent = Entry {
    meta,
    hash: parent_hash,
    children: parent_children,
  };

  Ok(parent)
}

// Read, validate and sort directory entries.
fn read_directory_entries(path: &Path, follow_symlinks: bool) -> Vec<Meta> {
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

    match Meta::new(&entry.path(), follow_symlinks) {
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
