use std::{
  fs::File,
  io::{self, Read},
  path::Path,
};

use sha2::{Digest, Sha256};

pub fn hash_file(path: &Path) -> io::Result<String> {
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
