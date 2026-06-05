use std::error::Error;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::dbpf::{ Dbpf, Identifier, TypeId };
use crate::dbpf::resource::DecodedResource;
use crate::helpers::{ get_package_paths_recursively, get_objd_related_resources, create_folder };

pub fn extract_objects(input: Option<PathBuf>, output: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
	let input = input.unwrap_or(PathBuf::from("./"));
	let output = output.unwrap_or(input.clone());

	let mut raw_resources = HashMap::new();

	println!("Reading packages...");
	let package_paths = get_package_paths_recursively(&input);
	for package_path in package_paths {
		println!("  {}", package_path.to_string_lossy());
		let resources = Dbpf::read_resources_from_file(&package_path)?;
		for resource in resources {
			match resource.id.type_id {
				TypeId::Objd | TypeId::Ctss | TypeId::Cres | TypeId::Shpe | TypeId::Gmnd | TypeId::Gmdc | TypeId::Mmat | TypeId::Txmt | TypeId::Txtr | TypeId::Lifo | TypeId::TextList => {
					raw_resources.insert(resource.id.clone(), resource);
				},
				_ => {}
			}
		}
	}
	println!("DONE");

	print!("Extracting object resources...");
	for (id, raw_resource) in &raw_resources {
		if id.type_id == TypeId::Objd {
			if let DecodedResource::Objd(objd) = raw_resource.decode()? {
				if (objd.multi_tile_master_id == 0 || objd.multi_tile_sub_index == 0xffff) && objd.function_sort_string() != "Unknown" {
					let ctss_id = Identifier {
						type_id: TypeId::Ctss,
						group_id: objd.id.group_id,
						instance_id: objd.catalog_strings_id as u32,
						resource_id: 0
					};
					if let Some(Ok(DecodedResource::Ctss(ctss))) = raw_resources.get(&ctss_id).map(|r| r.decode()) {
						if let Some(english_text) = ctss.text_list.get_items_by_language(1).first() {
							let object_name = make_alphanumeric(&String::from_utf8_lossy(&objd.file_name));
							let catalog_name = make_alphanumeric(&english_text.title);
							if !catalog_name.is_empty() {
								let file_name = format!("{object_name}_{catalog_name}_{:08x}.package", objd.guid);
								let output_folder = create_folder(&output, &objd.function_sort_string())?;
								let output_path = output_folder.join(file_name);
								println!("  {}", output_path.to_string_lossy());
								let resources = get_objd_related_resources(&objd, &raw_resources)?;
								Dbpf::write_package_file(&resources, &output_path)?;
							}
						}
					};
				}
			}
		}
	}
	println!("DONE");

	Ok(())
}

fn make_alphanumeric(s: &str) -> String {
	let mut s2 = String::new();
	for c in s.chars() {
		if c.is_alphanumeric() {
			s2.push(c);
		}
	}
	s2
}
