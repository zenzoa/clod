use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::path::{ Path, PathBuf };

use crate::dbpf::Dbpf;

pub mod extract_outfits;
pub mod extract_hairs;

pub fn get_skin_packages(path: &Path) -> Result<Vec<Dbpf>, Box<dyn Error>> {
	let mut dir_entries: Vec<DirEntry> = fs::read_dir(path)?
		.filter_map(|entry|
			match entry {
				Ok(entry) => Some(entry),
				Err(_) => None
			}).collect();

	dir_entries.sort_by_key(|entry| entry.file_name().to_string_lossy().into_owned());

	let mut packages = Vec::new();

	for dir_entry in dir_entries {
		let entry_path = dir_entry.path();
		if entry_path.is_file() && entry_path.extension().is_some_and(|e| e == "package") {
			let dbpf = Dbpf::read_from_file(&entry_path, &entry_path.file_stem().unwrap().to_string_lossy())?;
			packages.push(dbpf);
		}
	}

	Ok(packages)
}

fn create_folder(output_path: &Path, folder_name: &str) -> Result<PathBuf, Box<dyn Error>>{
	let folder_path = output_path.join(folder_name);
	if !folder_path.is_dir() {
		fs::create_dir(&folder_path)?;
	}
	Ok(folder_path)
}
