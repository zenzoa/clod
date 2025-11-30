use std::error::Error;
use std::fs::{ self, File };
use std::io::{ self, Cursor, Write };
use std::path::PathBuf;

use crate::dbpf::Dbpf;

pub fn compress_packages(files: Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
	for file in files {
		if file.is_file() && file.extension().is_some_and(|e| e == "package") {
			print!("Compressing {}...", file.to_string_lossy());
			io::stdout().flush()?;

			// make backup copy
			fs::copy(&file, file.with_extension("package.bak"))?;

			// read package file
			let bytes = fs::read(&file)?;
			let (resources, header, _) = Dbpf::read_resources(&bytes)?;

			// save package file with compression
			let mut cur = Cursor::new(Vec::new());
			Dbpf::write_resources(resources, header, &mut cur, true)?;
			let mut new_file = File::create(&file)?;
			new_file.write_all(&cur.into_inner())?;

			println!(" DONE");
		}
	}
	Ok(())
}
