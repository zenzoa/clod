use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use std::collections::HashMap;

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Category, Part };
use crate::dbpf::resource_types::idr::Idr;

pub fn save_skins(path: &Path, new_path: &Path) -> Result<(), Box<dyn Error>> {
	let mut outfits: HashMap<String, (Gzps, Idr)> = HashMap::new();

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
					if let Some(idr) = idr_list
						.iter()
						.find(|i| (i.id.group_id, i.id.instance_id, i.id.resource_id) == (gzps.id.group_id, gzps.id.instance_id, gzps.id.resource_id)) {
							outfits.insert(gzps.name.to_string(), (gzps, idr.clone()));
						}
				}
			}
		}
	}

	let mut outfit_groups: HashMap<String, Vec<DecodedResource>> = HashMap::new();

	for (gzps, idr) in outfits.values() {
		if !(gzps.species == 1 &&
			!gzps.category.contains(&Category::Skin) &&
			!gzps.category.contains(&Category::TryOn) &&
			!gzps.category.contains(&Category::Overlay) &&
			gzps.parts.len() == 1 &&
			(gzps.parts[0] == Part::Body || gzps.parts[0] == Part::Top || gzps.parts[0] == Part::Bottom)) {
				continue
		}

		let group_name = gzps.generate_name();

		match outfit_groups.get_mut(&group_name) {
			Some(resources) => {
				resources.push(DecodedResource::Gzps(gzps.clone()));
				resources.push(DecodedResource::Idr(idr.clone()));
			},
			None => {
				outfit_groups.insert(group_name.to_string(), vec![
					DecodedResource::Gzps(gzps.clone()),
					DecodedResource::Idr(idr.clone()),
				]);
			}
		}
	}

	let mut outfit_group_names: Vec<&String> = outfit_groups.keys().collect();
	outfit_group_names.sort();

	for name in outfit_group_names {
		if let Some(resources) = outfit_groups.get(name) {
			let mut file_path = new_path.to_path_buf();
			file_path.push(format!("{name}.package"));
			Dbpf::write_package_file(resources, &file_path.to_string_lossy(), false)?;
		}
	}

	Ok(())
}
