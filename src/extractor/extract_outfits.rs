use std::error::Error;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::dbpf::{ Dbpf, TypeId };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Category, Part };
use crate::dbpf::resource_types::idr::Idr;

use super::get_skin_packages;

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
		if outfit.gzps.species == 1 &&
			!outfit.gzps.category.contains(&Category::Skin) &&
			!outfit.gzps.category.contains(&Category::TryOn) &&
			!outfit.gzps.category.contains(&Category::Overlay) &&
			outfit.gzps.parts.len() == 1 &&
			(outfit.gzps.parts[0] == Part::Body || outfit.gzps.parts[0] == Part::Top || outfit.gzps.parts[0] == Part::Bottom) {
				sort_into_group(outfit, &mut outfit_groups, &outfit.gzps.generate_key());
		}
	}

	let mut outfit_group_names: Vec<&String> = outfit_groups.keys().collect();
	outfit_group_names.sort();

	for name in outfit_group_names {
		if let Some(resources) = outfit_groups.get(name) {
			let count = resources.iter().filter(|r| r.get_id().type_id == TypeId::Gzps).count();
			let mut file_path = output_path.to_path_buf();
			file_path.push(format!("{name}_{count}.package"));
			Dbpf::write_package_file(resources, &file_path, false)?;
		}
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
			let key = if gzps.parts.contains(&Part::Hair) { gzps.generate_key() } else { gzps.name.to_string() };
			outfits.insert(key, OriginalOutfit{ gzps, idr });
		}
	}

	outfits
}
