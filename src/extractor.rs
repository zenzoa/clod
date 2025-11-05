use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::path::{ Path, PathBuf };
use std::collections::HashMap;

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Category, Part };
use crate::dbpf::resource_types::idr::Idr;

struct OriginalOutfit {
	gzps: Gzps,
	idr: Option<Idr>
}

pub fn extract_outfits(input_path: Option<PathBuf>, output_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
	let input_path = input_path.unwrap_or(PathBuf::from("./"));
	let output_path = output_path.unwrap_or(input_path.clone());

	let packages = get_skin_packages(&input_path)?;
	let outfits = get_outfits(&packages);

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
		sort_into_group(outfit, &mut outfit_groups, &outfit.gzps.generate_name());
	}

	let mut outfit_group_names: Vec<&String> = outfit_groups.keys().collect();
	outfit_group_names.sort();

	for name in outfit_group_names {
		if let Some(resources) = outfit_groups.get(name) {
			let mut file_path = output_path.to_path_buf();
			file_path.push(format!("{name}.package"));
			Dbpf::write_package_file(resources, &file_path, false)?;
		}
	}

	Ok(())
}

pub fn extract_hairs(input_path: Option<PathBuf>, output_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
	let input_path = input_path.unwrap_or(PathBuf::from("./"));
	let output_path = output_path.unwrap_or(input_path.clone());

	let packages = get_skin_packages(&input_path)?;
	let hairs = get_outfits(&packages);

	let mut visible_hairs: HashMap<String, Vec<DecodedResource>> = HashMap::new();
	let mut hidden_hairs: HashMap<String, Vec<DecodedResource>> = HashMap::new();
	for hair in hairs.values() {
		if hair.gzps.species == 1 &&
			hair.gzps.parts.len() == 1 && hair.gzps.parts[0] == Part::Hair &&
			!hair.gzps.category.contains(&Category::Skin) {
				if hair.gzps.flags & 1 == 0 {
					sort_into_group(hair, &mut visible_hairs, &hair.gzps.generate_name());
				} else {
					sort_into_group(hair, &mut hidden_hairs, &hair.gzps.generate_name());
				}
		}
	}

	for (name, resources) in visible_hairs {
		if resources.len() == 1 {
			if let DecodedResource::Gzps(gzps) = &resources[0] {
				let folder_name = gzps.generate_hair_folder_name();
				let mut folder_path = output_path.clone();
				folder_path.push(folder_name.clone());
				if !folder_path.is_dir() {
					fs::create_dir(&folder_path)?;
				}
				let mut file_path = folder_path.to_path_buf();
				file_path.push(format!("{name}.package"));
				Dbpf::write_package_file(&resources, &file_path, false)?;
			}
		}
	}

	for (name, resources) in hidden_hairs {
		let mut file_path = output_path.clone();
		file_path.push("_hidden");
		file_path.push(format!("{name}.package"));
		Dbpf::write_package_file(&resources, &file_path, false)?;
	}

	Ok(())
}

pub fn find_hairs(input_path: Option<PathBuf>, family: String) -> Result<(), Box<dyn Error>> {
	let input_path = input_path.unwrap_or(PathBuf::from("./"));

	let packages = get_skin_packages(&input_path)?;
	let hairs = get_outfits(&packages);

	let mut hair_groups: HashMap<String, Vec<DecodedResource>> = HashMap::new();
	for hair in hairs.values() {
		if hair.gzps.species == 1 && hair.gzps.parts.len() == 1 && hair.gzps.parts[0] == Part::Hair && hair.gzps.family.to_string() == family {
			sort_into_group(hair, &mut hair_groups, &hair.gzps.name.to_string());
		}
	}

	let mut hair_group_names: Vec<&String> = hair_groups.keys().collect();
	hair_group_names.sort();
	for name in &hair_group_names {
		println!("{name}");
	}

	Ok(())
}

fn sort_into_group(outfit: &OriginalOutfit, groups: &mut HashMap<String, Vec<DecodedResource>>, group_name: &str) {
	match groups.get_mut(group_name) {
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
			groups.insert(group_name.to_string(), resources);
		}
	}
}

fn get_outfits(packages: &[Dbpf]) -> HashMap<String, OriginalOutfit> {
	let mut outfits: HashMap<String, OriginalOutfit> = HashMap::new();

	for package in packages {
		let gzps_list: Vec<Gzps> = package.resources
			.iter()
			.filter_map(|r| if let DecodedResource::Gzps(gzps) = r { Some(gzps.clone()) } else { None })
			.collect();
		let idr_list: Vec<Idr> = package.resources
			.iter()
			.filter_map(|r| if let DecodedResource::Idr(idr) = r { Some(idr.clone()) } else { None })
			.collect();
		for gzps in gzps_list {
			let idr = idr_list
				.iter()
				.find(|i| (i.id.group_id, i.id.instance_id, i.id.resource_id) == (gzps.id.group_id, gzps.id.instance_id, gzps.id.resource_id))
				.cloned();
			let key = if gzps.parts.contains(&Part::Hair) { gzps.generate_name() } else { gzps.name.to_string() };
			outfits.insert(key, OriginalOutfit{ gzps, idr });
		}
	}

	outfits
}

fn get_skin_packages(path: &Path) -> Result<Vec<Dbpf>, Box<dyn Error>> {
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
