use std::{
  fs::{self, File, read_dir},
  io::{self, Read},
  path::Path,
};

use cached::macros::cached;

use sha2::{Digest, Sha256};

#[cached(max_size = 1000, ttl_secs = 60)]
pub fn hash_file(path: &Path) -> io::Result<String> {
  let mut file = File::open(path)?;
  let mut hasher = Sha256::new();
  let mut buffer = [0; 4096];

  let file_name = match path.file_name() {
    Some(val) => val.to_string_lossy().into_owned(),
    None => String::from(""),
  };

  // Add type marker
  hasher.update(b"file");

  // Add file name size
  hasher.update((file_name.len() as u64).to_le_bytes());

  // Add file name
  hasher.update(file_name);

  loop {
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read == 0 {
      break;
    }

    hasher.update(&buffer[..bytes_read]);
  }

  Ok(hex::encode(hasher.finalize()))
}

#[cached(max_size = 1000, ttl_secs = 60)]
pub fn hash_folder(path: &Path) -> io::Result<String> {
  let entries = read_dir(path)?;

  let mut hasher = Sha256::new();

  for entry in entries {
    let entry = entry?;

    let metadata = fs::metadata(entry.path())?;

    if metadata.is_file() {
      hasher.update(hash_file(&entry.path())?);
    } else if metadata.is_dir() {
      hasher.update(hash_folder(&entry.path())?);
    }
  }

  Ok(hex::encode(hasher.finalize()))
}
