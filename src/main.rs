mod meta;

use clap::Parser;
use meta::Meta;
use std::{fs::File, path::PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(long)]
  json: bool,

  #[arg(long, short)]
  recursive: bool,

  #[arg(required = true, num_args = 1..)]
  paths: Vec<PathBuf>,
}

fn process_files(files: Vec<(File, String)>) -> Vec<Meta> {
  let mut metadata_list: Vec<Meta> = Vec::new();

  for file_tuple in files {
    let Ok(metadata) = Meta::new(&file_tuple.0, &file_tuple.1) else {
      println!(
        "An error occurred while trying to get the metadata for the following file: {}",
        file_tuple.1
      );

      continue;
    };

    metadata_list.push(metadata);
  }

  metadata_list
}

fn main() {
  let args = Args::parse();

  if args.json {
    println!("Json mode activated!");
  }

  if args.recursive {
    println!("Recursive mode activated!")
  }

  for path in &args.paths {
    println!("Path: {}", path.display());
  }
}
