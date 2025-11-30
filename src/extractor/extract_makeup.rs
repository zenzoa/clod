use std::error::Error;
use std::path::PathBuf;

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;

use super::get_skin_packages;

pub fn extract_makeup(input_path: Option<PathBuf>, output_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
	let input_path = input_path.unwrap_or(PathBuf::from("./"));
	let output_path = output_path.unwrap_or(input_path.clone());

	let packages = get_skin_packages(&input_path)?;
	for package in packages {
		for resource in &package.resources {
			if let DecodedResource::Xtol(xtol) = resource {
				if xtol.species == 1 {
					for resource2 in &package.resources {
						if let DecodedResource::Idr(idr) = resource2 {
							if idr.id.group_id == xtol.id.group_id &&
								idr.id.instance_id == xtol.id.instance_id &&
								idr.id.resource_id == xtol.id.resource_id {
									let resources = vec![
										DecodedResource::Xtol(xtol.clone()),
										DecodedResource::Idr(idr.clone())
									];
									print!("Extracting {}...", xtol.name);
									let file_path = output_path.join(format!("{}.package", xtol.name));
									Dbpf::write_package_file(&resources, &file_path, false)?;
									println!("DONE");
							}
						}
					}
				}
			}
		}
	}

	Ok(())
}
