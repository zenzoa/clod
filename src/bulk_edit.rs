use std::error::Error;
use std::fs;
use std::io::{ self, Write };
use std::path::PathBuf;

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;

pub fn edit_gzps(files: Vec<PathBuf>, property: &str, value: &str) -> Result<(), Box<dyn Error>> {
	for file in files {
		if file.is_file() && file.extension().is_some_and(|e| e == "package") {
			print!("Editing {}...", file.to_string_lossy());
			io::stdout().flush()?;

			// make backup copy
			fs::copy(&file, file.with_extension("package.bak"))?;

			// read package file
			let mut package = Dbpf::read_from_file(&file, "")?;

			// change property value in all GZPS
			for resource in package.resources.iter_mut() {
				if let DecodedResource::Gzps(gzps) = resource {
					gzps.set_property(property, value)?;
				}
			}

			// save package file
			package.write_to_file(&file)?;

			println!(" DONE");
		}
	}
	Ok(())
}
