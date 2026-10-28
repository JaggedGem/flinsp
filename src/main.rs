use std::path::PathBuf;

use clap::{Parser};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    json: bool,

    #[arg(long, short)]
    recursive: bool,

    #[arg(required = true, num_args = 1..)]
    paths: Vec<PathBuf>
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
