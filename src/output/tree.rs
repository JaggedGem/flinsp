use chrono::{DateTime, Local};

use crate::scanner::Entry;

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

fn print_information(entry: &Entry, ancestors: &[bool], is_last: bool, is_root: bool) {
  let branch = branch_prefix(ancestors, is_last, is_root);
  let prefix = detail_prefix(ancestors, is_last, is_root);
  let spacing = "Last Modified:".len() + 2;

  let metadata = &entry.meta;

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

  if let Some(hash) = &entry.hash {
    println!("{prefix}{:<spacing$}{hash}", "File Hash:");
  }
}

// Render an already collected directory's children.
fn process_folder_recursive(entries: &[Entry], is_recursive: bool, ancestors: &[bool]) {
  let total = entries.len();

  for (index, entry) in entries.iter().enumerate() {
    let is_last = index + 1 == total;

    if index > 0 {
      println!("{}", tree_spine(ancestors));
    }

    print_information(entry, ancestors, is_last, false);

    if let Some(children) = &entry.children {
      if !children.is_empty() && is_recursive {
        let mut child_ancestors = ancestors.to_vec();

        child_ancestors.push(!is_last);

        println!("{}", tree_spine(&child_ancestors));

        process_folder_recursive(children, is_recursive, &child_ancestors);
      } else if is_last {
        println!("{}", closing_prefix(ancestors, is_last, false));
      }
    } else if is_last {
      println!("{}", closing_prefix(ancestors, is_last, false));
    }
  }
}

pub(crate) fn print_tree(roots: &[Entry], _include_hashes: bool, is_recursive: bool) {
  for (index, entry) in roots.iter().enumerate() {
    if index > 0 {
      println!();
    }

    print_information(entry, &[], true, true);

    if let Some(children) = &entry.children {
      if !children.is_empty() && is_recursive {
        println!("│");

        process_folder_recursive(children, is_recursive, &[]);
      } else {
        println!("{}", closing_prefix(&[], true, true));
      }
    } else {
      println!("{}", closing_prefix(&[], true, true));
    }
  }
}
