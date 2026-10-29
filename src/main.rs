mod meta;

use chrono::{DateTime, Local};
use clap::Parser;
use meta::Meta;
use sha2::{Digest, Sha256};
use std::{
  collections::VecDeque,
  ffi::OsString,
  fs::{self, File, FileType},
  io::{self, Error, Read},
  path::{self, Path},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(long)]
  json: bool,

  #[arg(long, short)]
  recursive: bool,

  #[arg(required = true, num_args = 1..)]
  paths: Vec<path::PathBuf>,

  #[arg(long = "include-hashes", short)]
  include_hashes: bool,
}

fn print_information(metadata: &Meta, hash: Option<String>) {
  let spacing = "last modified:".len() + 2;

  println!(
    "{} Information",
    if metadata.kind.is_file() {
      "File"
    } else {
      "Folder"
    }
  );
  println!("-------------------------------------------");

  println!(
    "{:<spacing$}{}{}",
    "Name:",
    &metadata.name.display(),
    if metadata.kind.is_file() {
      format!(" ({}.{})", metadata.name.display(), metadata.extension)
    } else {
      String::new()
    },
  );

  if metadata.kind.is_file() {
    println!(
      "{:<spacing$}{}",
      "Extension:",
      format_args!(".{}", metadata.extension)
    );
  }

  println!(
    "{:<spacing$}{} ({} Bytes)",
    "Size:",
    metadata.human_readable_size(),
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
    Err(_) => metadata.path.clone(),
  };

  println!("{:<spacing$}{}", "Path:", absolute_path.display());

  if let Some(val) = hash {
    println!("{:<spacing$}{}", "File Hash:", val)
  }
}

fn hash_file(path: &Path) -> io::Result<String> {
  let mut file = File::open(path)?;
  let mut hasher = Sha256::new();
  let mut buffer = [0; 4096]; // Read in 4KB chunks

  loop {
    let bytes_read = file.read(&mut buffer)?;
    if bytes_read == 0 {
      break; // End of file
    }
    hasher.update(&buffer[..bytes_read]);
  }

  // Finalize the hash and format as a hex string
  Ok(hex::encode(hasher.finalize()))
}

fn process_entry(path: &Path, include_hashes: bool) -> Result<FileType, Error> {
  let metadata = match Meta::new(path) {
    Ok(val) => val,
    Err(error) => {
      eprintln!(
        "An error occurred while trying to get the metadata for a file ({}): {}",
        match path.file_name() {
          Some(val) => val.to_owned(),
          None => OsString::from(""),
        }
        .display(),
        error
      );

      return Err(error);
    }
  };

  // Print the information
  print_information(
    &metadata,
    if metadata.kind.is_file() && include_hashes {
      match hash_file(path) {
        Ok(val) => Some(val),
        Err(error) => {
          eprintln!(
            "An error occurred while trying to get the hash ({}): {}",
            path.display(),
            error
          );

          None
        }
      }
    } else {
      None
    },
  );
  print!("\n\n");

  Ok(metadata.kind)
}

fn process_folder_recursive(path: &Path, include_hashes: bool) {
  let entries = match match fs::read_dir(path) {
    Ok(val) => val,
    Err(error) => {
      eprintln!(
        "An error occurred while trying to read a folder ({}): {}",
        path.display(),
        error
      );

      return;
    }
  }
  .collect::<Result<Vec<_>, _>>()
  {
    Ok(val) => val,
    Err(error) => {
      eprintln!(
        "An error occurred while trying to read a folder ({}): {}",
        path.display(),
        error
      );

      return;
    }
  };

  // entries.sort_by_key(|entry| entry.file_name());

  let mut dir_queue: VecDeque<Meta> = VecDeque::new();

  for entry in entries {
    let metadata = match Meta::new(&entry.path()) {
      Ok(val) => val,
      Err(error) => {
        eprintln!(
          "An error occurred while trying to get the metadata for a file in a subfolder ({}): {}",
          format_args!("{}/{}", entry.path().display(), entry.file_name().display()),
          error
        );

        continue;
      }
    };

    if metadata.kind.is_dir() {
      dir_queue.push_back(metadata);

      // Process the directories at the end
      continue;
    }

    // Print the information
    print_information(
      &metadata,
      if metadata.kind.is_file() && include_hashes {
        match hash_file(&metadata.path) {
          Ok(val) => Some(val),
          Err(error) => {
            println!(
              "An error occurred while trying to get the hash ({}): {}",
              metadata.path.display(),
              error
            );

            None
          }
        }
      } else {
        None
      },
    );

    print!("\n\n");
  }

  loop {
    if dir_queue.is_empty() {
      break;
    }

    let metadata = match dir_queue.pop_front() {
      Some(val) => val,
      None => break,
    };

    // Print the information
    print_information(&metadata, None);

    print!("\n\n");

    // Traverse the subfolders
    process_folder_recursive(&metadata.path, include_hashes);
  }
}

fn main() {
  let mut args = Args::parse();

  args.paths.sort();

  for path in args.paths {
    // Process every path provided and save their kind
    let kind = match process_entry(&path, args.include_hashes) {
      Ok(val) => val,
      Err(_) => continue,
    };

    //
    if kind.is_dir() && args.recursive {
      process_folder_recursive(&path, args.include_hashes);
    }
  }
}
