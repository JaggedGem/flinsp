use chrono::{DateTime, Local};

use crate::{hash::hash_file, meta::Meta, scanner::read_directory_entries};

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

  println!("{prefix}{:<spacing$}{}", "Path:", metadata.path.display());

  if let Some(hash) = hash {
    println!("{prefix}{:<spacing$}{hash}", "File Hash:");
  }
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

pub(crate) fn print_tree(roots: &[Meta], include_hashes: bool, is_recursive: bool) {
  for (index, metadata) in roots.iter().enumerate() {
    if index > 0 {
      println!();
    }

    // Root always starts at column 0.
    process_entry(metadata, include_hashes, &[], true, true);

    let children = if is_recursive && metadata.kind.is_dir() {
      read_directory_entries(&metadata.path)
    } else {
      Vec::new()
    };

    if !children.is_empty() {
      // Extend the root's vertical line to its children.
      println!("│");

      process_folder_recursive(&children, is_recursive, &[]);
    } else {
      // A standalone file, empty folder, or non-recursive
      // root is also a terminal leaf.
      println!("{}", closing_prefix(&[], true, true));
    }
  }
}
