mod meta;

use chrono::{DateTime, Local};
use clap::Parser;
use meta::Meta;
use sha2::{Digest, Sha256};
use std::{
  fs::{self, File},
  io::{self, Read},
  path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(long)]
  json: bool,

  #[arg(long, short)]
  recursive: bool,

  #[arg(required = true, num_args = 1..)]
  paths: Vec<PathBuf>,

  #[arg(long = "include-hashes", short)]
  include_hashes: bool,
}

// One ancestor occupies exactly three terminal columns.
fn ancestor_prefix(ancestors: &[bool]) -> String {
  let mut prefix = String::new();

  for &has_next in ancestors {
    prefix.push_str(if has_next { "│  " } else { "   " });
  }

  prefix
}

// Root starts at column zero.
// Every heading has a junction for its metadata.
fn branch_prefix(ancestors: &[bool], is_last: bool, is_root: bool) -> String {
  if is_root {
    return "┌──┬── ".to_string();
  }

  let mut prefix = ancestor_prefix(ancestors);

  prefix.push_str(if is_last {
    "└──┬── "
  } else {
    "├──┬── "
  });

  prefix
}

// Align the metadata line with the heading's ┬.
fn detail_prefix(ancestors: &[bool], is_last: bool, is_root: bool) -> String {
  let mut prefix = ancestor_prefix(ancestors);

  prefix.push_str(if is_root || !is_last {
    "│  │   "
  } else {
    "   │   "
  });

  prefix
}

// Close the metadata's vertical line at its own column.
fn closing_prefix(ancestors: &[bool], is_last: bool, is_root: bool) -> String {
  let mut prefix = ancestor_prefix(ancestors);

  prefix.push_str(if is_root || !is_last {
    "│  └──"
  } else {
    "   └──"
  });

  prefix
}

// Continue the tree between siblings or into children.
fn tree_spine(ancestors: &[bool]) -> String {
  format!("{}│", ancestor_prefix(ancestors))
}

fn print_information(
  metadata: &Meta,
  hash: Option<&str>,
  ancestors: &[bool],
  is_last: bool,
  is_root: bool,
) {
  let branch = branch_prefix(ancestors, is_last, is_root);
  let prefix = detail_prefix(ancestors, is_last, is_root);
  let spacing = "Last Modified:".len() + 2;

  let kind = if metadata.kind.is_dir() {
    "Folder"
  } else {
    "File"
  };

  println!("{branch}{kind} Information");
  println!("{prefix}-------------------------------------------");

  let name = if metadata.kind.is_file() && !metadata.extension.is_empty() {
    format!("{}.{}", metadata.name.display(), metadata.extension)
  } else {
    metadata.name.display().to_string()
  };

  println!("{prefix}{:<spacing$}{name}", "Name:");

  if metadata.kind.is_file() {
    let extension = if metadata.extension.is_empty() {
      String::from("(none)")
    } else {
      format!(".{}", metadata.extension)
    };

    println!("{prefix}{:<spacing$}{extension}", "Extension:");
  }

  println!(
    "{prefix}{:<spacing$}{} ({} Bytes)",
    "Size:",
    metadata.human_readable_size(),
    metadata.size
  );

  if let Some(time) = metadata.last_modified {
    let local: DateTime<Local> = time.into();

    println!(
      "{prefix}{:<spacing$}{}",
      "Last Modified:",
      local.format("%Y-%m-%d %H:%M:%S")
    );
  }

  if let Some(time) = metadata.last_accessed {
    let local: DateTime<Local> = time.into();

    println!(
      "{prefix}{:<spacing$}{}",
      "Last Accessed:",
      local.format("%Y-%m-%d %H:%M:%S")
    );
  }

  if let Some(time) = metadata.created_at {
    let local: DateTime<Local> = time.into();

    println!(
      "{prefix}{:<spacing$}{}",
      "Created At:",
      local.format("%Y-%m-%d %H:%M:%S")
    );
  }

  let absolute_path = std::path::absolute(&metadata.path).unwrap_or_else(|_| metadata.path.clone());

  println!("{prefix}{:<spacing$}{}", "Path:", absolute_path.display());

  if let Some(hash) = hash {
    println!("{prefix}{:<spacing$}{hash}", "File Hash:");
  }
}

fn hash_file(path: &Path) -> io::Result<String> {
  let mut file = File::open(path)?;
  let mut hasher = Sha256::new();
  let mut buffer = [0; 4096];

  loop {
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read == 0 {
      break;
    }

    hasher.update(&buffer[..bytes_read]);
  }

  Ok(hex::encode(hasher.finalize()))
}

fn process_entry(
  metadata: &Meta,
  include_hashes: bool,
  ancestors: &[bool],
  is_last: bool,
  is_root: bool,
) {
  let hash = if metadata.kind.is_file() && include_hashes {
    match hash_file(&metadata.path) {
      Ok(value) => Some(value),
      Err(error) => {
        eprintln!("Failed to hash {}: {}", metadata.path.display(), error);

        None
      }
    }
  } else {
    None
  };

  print_information(metadata, hash.as_deref(), ancestors, is_last, is_root);
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

// Render an already collected directory's children.
fn process_folder_recursive(entries: &[Meta], include_hashes: bool, ancestors: &[bool]) {
  let total = entries.len();

  for (index, metadata) in entries.iter().enumerate() {
    let is_last = index + 1 == total;

    // Maintain the existing tree connector between siblings.
    if index > 0 {
      println!("{}", tree_spine(ancestors));
    }

    process_entry(metadata, include_hashes, ancestors, is_last, false);

    // Discover children before drawing their connector.
    // This avoids dangling vertical lines in empty folders.
    let children = if metadata.kind.is_dir() {
      read_directory_entries(&metadata.path)
    } else {
      Vec::new()
    };

    if !children.is_empty() {
      let mut child_ancestors = ancestors.to_vec();

      // If this folder isn't the final sibling,
      // preserve its sibling connector.
      child_ancestors.push(!is_last);

      // Continue the folder's metadata junction into
      // the branches of its children.
      println!("{}", tree_spine(&child_ancestors));

      process_folder_recursive(&children, include_hashes, &child_ancestors);
    } else if is_last {
      // This is the final visible leaf.
      // Close its metadata line.
      println!("{}", closing_prefix(ancestors, is_last, false));
    }
  }
}

fn main() {
  let mut args = Args::parse();

  args.paths.sort();

  // Collect valid roots first.
  let mut roots = Vec::new();

  for path in &args.paths {
    match Meta::new(path) {
      Ok(metadata) => roots.push(metadata),
      Err(error) => {
        eprintln!("Failed to get metadata for {}: {}", path.display(), error);
      }
    }
  }

  for (index, metadata) in roots.iter().enumerate() {
    if index > 0 {
      println!();
    }

    // Root always starts at column 0.
    process_entry(metadata, args.include_hashes, &[], true, true);

    let children = if args.recursive && metadata.kind.is_dir() {
      read_directory_entries(&metadata.path)
    } else {
      Vec::new()
    };

    if !children.is_empty() {
      // Extend the root's vertical line to its children.
      println!("│");

      process_folder_recursive(&children, args.include_hashes, &[]);
    } else {
      // A standalone file, empty folder, or non-recursive
      // root is also a terminal leaf.
      println!("{}", closing_prefix(&[], true, true));
    }
  }
}
