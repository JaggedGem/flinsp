mod meta;

use chrono::{DateTime, Local};
use clap::Parser;
use meta::{EntryKind, Meta};
use std::path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(long)]
  json: bool,

  #[arg(long, short)]
  recursive: bool,

  #[arg(required = true, num_args = 1..)]
  paths: Vec<path::PathBuf>,
}

fn print_information(metadata: Meta) {
  let is_file: bool = match metadata.kind {
    EntryKind::File => true,
    EntryKind::Folder => false,
  };

  let spacing = "last modified:".len() + 2;

  println!("{} Information", if is_file { "File" } else { "Folder" });
  println!("-------------------------------------------");

  println!(
    "{:<spacing$}{}{}",
    "Name:",
    &metadata.name,
    if is_file {
      format!(" ({}.{})", metadata.name, metadata.extension)
    } else {
      String::new()
    },
  );

  if is_file {
    println!(
      "{:<spacing$}{}",
      "Extension:",
      String::from('.') + metadata.extension.as_str()
    );
  }

  println!(
    "{:<spacing$}{} ({} Bytes)",
    "Size:",
    metadata.get_human_readable_size(),
    metadata.size
  );

  if let Some(time) = metadata.last_modified {
    let last_modified_local: DateTime<Local> = time.into();

    println!(
      "{:<spacing$}{}",
      "Last Modified:",
      last_modified_local.format("%Y-%m-%d %H:%M:%S")
    );
  }

  if let Some(time) = metadata.last_accessed {
    let last_accessed_local: DateTime<Local> = time.into();

    println!(
      "{:<spacing$}{}",
      "Last Accessed:",
      last_accessed_local.format("%Y-%m-%d %H:%M:%S")
    );
  }

  if let Some(time) = metadata.created_at {
    let created_at_local: DateTime<Local> = time.into();

    println!(
      "{:<spacing$}{}",
      "Created At:",
      created_at_local.format("%Y-%m-%d %H:%M:%S")
    );
  }

  let absolute_path = match path::absolute(metadata.path.as_path()) {
    Ok(val) => val,
    Err(_) => metadata.path,
  };

  println!("{:<spacing$}{}", "Path:", absolute_path.display());
}

fn main() {
  let args = Args::parse();

  for path in args.paths {
    let Ok(metadata) = Meta::new(&path) else {
      println!(
        "An error occurred while trying to get the metadata for the following file: {}",
        path.display()
      );

      continue;
    };

    // Print the information
    print_information(metadata);
    print!("\n\n")
  }
}
