use std::error::Error;
use std::io::{ Cursor, Write };
use std::fs::{ self, File };
use std::path::PathBuf;

use crate::dbpf::Dbpf;

pub fn compress_packages(files: Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
	for file in files {
		if file.is_file() && file.extension().is_some_and(|e| e == "package") {
			// make backup copy
			fs::copy(&file, file.with_extension("package.bak"))?;

			// read package file
			let bytes = fs::read(&file)?;
			let dbpf = Dbpf::read(&bytes, "")?;

			// save package file with compression
			let mut cur = Cursor::new(Vec::new());
			dbpf.write(&mut cur, true)?;
			let mut new_file = File::create(file)?;
			new_file.write_all(&cur.into_inner())?;
		}
	}
	Ok(())
}
