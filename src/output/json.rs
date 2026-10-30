use std::{path::PathBuf, time::SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::Error;

use crate::{hash::hash_file, meta::Meta, scanner::read_directory_entries};

#[derive(Deserialize, Serialize)]
enum Kind {
  File,
  Folder,
}

#[derive(Deserialize, Serialize)]
struct Item {
  pub name: String,
  pub extension: String,
  pub size: u64,
  pub last_modified: Option<SystemTime>,
  pub last_accessed: Option<SystemTime>,
  pub created_at: Option<SystemTime>,
  pub kind: Kind,
  pub path: PathBuf,
  pub hash: Option<String>,
  pub children: Option<Vec<Item>>,
}

fn build_item(root: &Meta, include_hashes: bool, is_recursive: bool) -> Item {
  Item {
    name: root.name.to_string_lossy().to_string(),
    extension: root.extension.to_owned(),
    size: root.size,
    last_modified: root.last_modified,
    last_accessed: root.last_accessed,
    created_at: root.created_at,
    kind: if root.kind.is_file() {
      Kind::File
    } else {
      Kind::Folder
    },
    path: root.path.to_owned(),
    hash: if include_hashes {
      hash_file(&root.path).ok()
    } else {
      None
    },
    children: if root.kind.is_dir() && is_recursive {
      let entries = read_directory_entries(&root.path);

      let mut children: Vec<Item> = Vec::new();
      for entry in entries {
        children.push(build_item(&entry, include_hashes, is_recursive));
      }

      Some(children)
    } else {
      None
    },
  }
}

pub(crate) fn print_json(
  roots: &[Meta],
  include_hashes: bool,
  is_recursive: bool,
) -> Result<(), Error> {
  let mut items: Vec<Item> = Vec::new();

  for metadata in roots {
    let item = build_item(metadata, include_hashes, is_recursive);

    items.push(item);
  }

  let json = serde_json::to_string_pretty(&items)?;

  println!("{}", json);

  Ok(())
}
