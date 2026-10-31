use std::{path::PathBuf, time::SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::Error;

use crate::scanner::Entry;

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

fn build_item(entry: &Entry) -> Item {
  Item {
    name: entry.meta.name.to_string_lossy().to_string(),
    extension: entry.meta.extension.to_owned(),
    size: entry.meta.size,
    last_modified: entry.meta.last_modified,
    last_accessed: entry.meta.last_accessed,
    created_at: entry.meta.created_at,
    kind: if entry.meta.kind.is_file() {
      Kind::File
    } else {
      Kind::Folder
    },
    path: entry.meta.path.to_owned(),
    hash: entry.hash.clone(),
    children: match &entry.children {
      Some(val) => {
        let mut item_children: Vec<Item> = Vec::new();

        for child in val {
          item_children.push(build_item(child));
        }

        Some(item_children)
      }

      None => None,
    },
  }
}

pub(crate) fn print_json(entries: &[Entry]) -> Result<(), Error> {
  let mut items: Vec<Item> = Vec::new();

  for entry in entries {
    let item = build_item(entry);

    items.push(item);
  }

  let json = serde_json::to_string_pretty(&items)?;

  println!("{}", json);

  Ok(())
}
