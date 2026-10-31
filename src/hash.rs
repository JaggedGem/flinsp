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

// pub fn hash_folder(path: &Path) -> io::Result<String> {
//   let mut entries: Vec<PathBuf> = read_dir(path)?
//     .map(|entry| entry.map(|e| e.path()))
//     .collect::<io::Result<Vec<_>>>()?;

//   entries.sort_by(|a, b| a.is_dir().cmp(&b.is_dir()).then_with(|| a.cmp(b)));

//   let mut hasher = Sha256::new();

//   for entry in entries {
//     if entry.is_file() {
//       hasher.update(hash_file(&entry)?);
//     } else if entry.is_dir() {
//       hasher.update(hash_folder(&entry)?);
//     }
//   }

//   Ok(hex::encode(hasher.finalize()))
// }

pub fn hash_folder(entries: &[(&Path, &str)]) -> io::Result<String> {
  let mut hasher = Sha256::new();

  for entry in entries {
    let file_name = match entry.0.file_name() {
      Some(val) => val.to_string_lossy().to_string(),
      None => String::from(""),
    };

    hasher.update(b"folder");

    hasher.update((file_name.len() as u64).to_le_bytes());

    hasher.update(file_name.as_bytes());

    hasher.update(entry.1);
  }

  Ok(hex::encode(hasher.finalize()))
}
