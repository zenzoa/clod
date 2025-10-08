use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use std::collections::HashMap;

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Category, Part };
use crate::dbpf::resource_types::idr::Idr;

struct OriginalOutfit {
	gzps: Gzps,
	idr: Option<Idr>
}

pub fn save_skins(path: &Path, new_path: &Path) -> Result<(), Box<dyn Error>> {
	let mut outfits: HashMap<String, OriginalOutfit> = HashMap::new();

	let mut dir_entries: Vec<DirEntry> = fs::read_dir(path)?
		.filter_map(|entry|
			match entry {
				Ok(entry) => Some(entry),
				Err(_) => None
			}).collect();

	dir_entries.sort_by_key(|entry| entry.file_name().to_string_lossy().into_owned());

	for dir_entry in dir_entries {
		if let Ok(filename) = dir_entry.file_name().into_string() {
			if filename.ends_with(".package") {
				let bytes = fs::read(dir_entry.path())?;
				let dbpf = Dbpf::read(&bytes, &filename.replace(".package", ""))?;
				let gzps_list: Vec<Gzps> = dbpf.resources
					.iter()
					.filter_map(|r| if let DecodedResource::Gzps(gzps) = r { Some(gzps.clone()) } else { None })
					.collect();
				let idr_list: Vec<Idr> = dbpf.resources
					.iter()
					.filter_map(|r| if let DecodedResource::Idr(idr) = r { Some(idr.clone()) } else { None })
					.collect();
				for gzps in gzps_list {
					let idr = idr_list
						.iter()
						.find(|i| (i.id.group_id, i.id.instance_id, i.id.resource_id) == (gzps.id.group_id, gzps.id.instance_id, gzps.id.resource_id))
						.cloned();
					outfits.insert(gzps.name.to_string(), OriginalOutfit{ gzps, idr });
				}
			}
		}
	}

	let mut outfit_groups: HashMap<String, Vec<DecodedResource>> = HashMap::new();

	for outfit in outfits.values() {
		if !(outfit.gzps.species == 1 &&
			!outfit.gzps.category.contains(&Category::Skin) &&
			!outfit.gzps.category.contains(&Category::TryOn) &&
			!outfit.gzps.category.contains(&Category::Overlay) &&
			outfit.gzps.parts.len() == 1 &&
			(outfit.gzps.parts[0] == Part::Body || outfit.gzps.parts[0] == Part::Top || outfit.gzps.parts[0] == Part::Bottom)) {
				continue
		}

		let group_name = outfit.gzps.generate_name();

		match outfit_groups.get_mut(&group_name) {
			Some(resources) => {
				resources.push(DecodedResource::Gzps(outfit.gzps.clone()));
				if let Some(idr) = &outfit.idr {
					resources.push(DecodedResource::Idr(idr.clone()));
				}
			},
			None => {
				let mut resources = vec![DecodedResource::Gzps(outfit.gzps.clone())];
				if let Some(idr) = &outfit.idr {
					resources.push(DecodedResource::Idr(idr.clone()))
				}
				outfit_groups.insert(group_name.to_string(), resources);
			}
		}
	}

	let mut outfit_group_names: Vec<&String> = outfit_groups.keys().collect();
	outfit_group_names.sort();

	for name in outfit_group_names {
		if let Some(resources) = outfit_groups.get(name) {
			let mut file_path = new_path.to_path_buf();
			file_path.push(format!("{name}.package"));
			Dbpf::write_package_file(resources, &file_path, false)?;
		}
	}

	Ok(())
}
