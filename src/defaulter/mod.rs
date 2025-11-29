use std::error::Error;
use std::fs;
use std::path::{ Path, PathBuf };
use std::ffi::OsString;

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::Gzps;

pub mod default_outfit;
pub mod default_hair;

pub fn get_default_replacement_files(source_dir: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>), Box<dyn Error>> {
	let mut original_files = Vec::new();
	let mut replacement_files = Vec::new();
	for entry in (fs::read_dir(source_dir)?).flatten() {
		let entry_path = entry.path();
		if entry_path.is_file() && entry_path.extension().unwrap_or(&OsString::new()) == "package" {
			original_files.push(entry_path);
		} else if entry_path.is_dir() {
			for subentry in (fs::read_dir(entry_path)?).flatten() {
				let subentry_path = subentry.path();
				if subentry_path.is_file() && subentry_path.extension().unwrap_or(&OsString::new()) == "package" {
					replacement_files.push(subentry_path);
				}
			}
		}
	}
	original_files.sort();
	replacement_files.sort();
	Ok((original_files, replacement_files))
}

pub fn extract_resources(files: &[PathBuf]) -> Result<Vec<DecodedResource>, Box<dyn Error>> {
	let resources: Vec<DecodedResource> = files
		.iter()
		.map(|file| {
			let new_name = file.file_stem().map_or("UNKNOWN".to_string(), |x| x.to_string_lossy().into_owned());
			let dbpf = Dbpf::read_from_file(file, &new_name)?;
			Ok(dbpf.resources)
		})
		.collect::<Result<Vec<Vec<DecodedResource>>, Box<dyn Error>>>()?
		.into_iter()
		.flatten()
		.collect();
	Ok(resources)
}

pub fn extract_gzps(files: &[PathBuf]) -> Result<Vec<Gzps>, Box<dyn Error>> {
	let mut gzps_list = extract_resources(files)?
		.iter()
		.filter_map(|res|
			if let DecodedResource::Gzps(gzps) = res {
				Some(gzps.clone())
			} else {
				None
			})
		.collect::<Vec<_>>();
	gzps_list.sort_by_key(|gzps| gzps.name.to_string());
	Ok(gzps_list)
}

pub fn default_output_path(source_dir: &Path, suffix: &str) -> PathBuf {
	if let Ok(abs_path) = fs::canonicalize(source_dir) {
		let dir_name = abs_path.file_name().map(|s| s.to_string_lossy()).unwrap_or("".into());
		source_dir.join(PathBuf::from(format!("{dir_name}_{suffix}.package")))
	} else {
		source_dir.join("{suffix}.package")
	}
}
