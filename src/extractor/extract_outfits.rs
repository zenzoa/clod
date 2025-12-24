use std::error::Error;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Category, Part };
use crate::dbpf::resource_types::idr::Idr;

use super::{ get_skin_packages, create_folder };

#[derive(Clone)]
struct OriginalOutfit {
	gzps: Gzps,
	idr: Option<Idr>
}

pub fn extract_outfits(input_path: Option<PathBuf>, output_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
	let input_path = input_path.unwrap_or(PathBuf::from("./"));
	let output_path = output_path.unwrap_or(input_path.clone());

	print!("Reading Skin.package files...");
	let packages = get_skin_packages(&input_path)?;
	let outfits = get_outfits(&packages);
	println!("DONE");

	let mut outfit_groups: HashMap<String, Vec<&OriginalOutfit>> = HashMap::new();
	for outfit in outfits.values() {
		if outfit.gzps.species == 1 &&
			!outfit.gzps.categories.contains(&Category::Skin) &&
			!outfit.gzps.categories.contains(&Category::TryOn) &&
			!outfit.gzps.categories.contains(&Category::Overlay) &&
			outfit.gzps.parts.len() == 1 &&
			!outfit.gzps.name.to_string().contains("fried") &&
			(outfit.gzps.parts[0] == Part::Body || outfit.gzps.parts[0] == Part::Top || outfit.gzps.parts[0] == Part::Bottom) {
				let group_name = outfit.gzps.generate_key();
				if let Some(group) = outfit_groups.get_mut(&group_name) {
					group.extend_from_slice(&[outfit]);
				} else {
					outfit_groups.insert(group_name, vec![outfit]);
				}
		}
	}

	for (group_name, outfits) in outfit_groups {
		print!("Extracting {group_name}...");
		let folder_name = format!("{group_name}_{}", outfits.len());
		let folder_path = create_folder(&output_path, &folder_name)?;
		for outfit in outfits {
			let mut resources = vec![DecodedResource::Gzps(outfit.gzps.clone())];
			if let Some(idr) = &outfit.idr {
				resources.push(DecodedResource::Idr(idr.clone()));
			}
			let hidden = if outfit.gzps.flags & 1 > 0 { "_hidden" } else { "" };
			let file_name = format!("{}_{}{}", outfit.gzps.name, Category::stringify(&outfit.gzps.categories), hidden);
			let file_path = folder_path.join(file_name).with_extension("package");
			Dbpf::write_package_file(&resources, &file_path, false)?;
		}
		println!("DONE");
	}

	Ok(())
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
