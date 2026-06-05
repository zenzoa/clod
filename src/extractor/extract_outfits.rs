use std::error::Error;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Part };

use crate::helpers::{ ExtractedOutfit, create_folder, get_gzps_related_resources, get_packages_in_dir, get_resources_in_packages };

pub fn extract_outfits(input: Option<PathBuf>, output: Option<PathBuf>, bins: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
	let input = input.unwrap_or(PathBuf::from("./"));
	let output = output.unwrap_or(input.clone());

	print!("Reading Skins.package files...");
	let skin_packages = get_packages_in_dir(&input)?;
	let skin_resources = get_resources_in_packages(&skin_packages)?;
	println!("DONE");

	if bins.is_some() { print!("Reading globalcatbin.bundle.package files..."); }
	let bin_packages = match &bins {
		Some(bins_path) => get_packages_in_dir(bins_path)?,
		None => Vec::new()
	};
	let bin_resources = get_resources_in_packages(&bin_packages)?;
	if bins.is_some() { println!("DONE"); }

	print!("Looking for outfit GZPS resources...");
	let gzps_list = skin_resources.iter()
		.filter_map(|r|
			match r {
				DecodedResource::Gzps(gzps) =>
					if gzps.species == 1 && (gzps.parts.contains(&Part::Body) || gzps.parts.contains(&Part::Top) || gzps.parts.contains(&Part::Bottom)) {
						Some(gzps.clone())
					} else {
						None
					},
				_ => None
			})
		.collect::<Vec<Gzps>>();
	println!("DONE");

	if bins.is_some() {
		print!("Finding corresponding 3IDR resources...");
	} else {
		print!("Finding corresponding 3IDR and BINX resources...");
	}
	let mut outfit_groups: HashMap<String, Vec<ExtractedOutfit>> = HashMap::new();
	for gzps in gzps_list {
		let (gzps_idr, binx_idr, binx) = get_gzps_related_resources(&gzps.id, &skin_resources, &bin_resources);

		let group_name = gzps.outfit_group_name();
		let outfit = ExtractedOutfit {
			gzps,
			gzps_idr,
			binx,
			binx_idr
		};

		if let Some(outfit_group) = outfit_groups.get_mut(&group_name) {
			outfit_group.push(outfit);
		} else {
			outfit_groups.insert(group_name, vec![outfit]);
		}
	}
	println!("DONE");

	print!("Saving resources as new packages...");
	for (group_name, outfits) in outfit_groups.iter() {
		let group_path = create_folder(&output, &format!("{group_name}_{}", outfits.len()))?;
		for outfit in outfits {
			let file_name = group_path.join(outfit.gzps.outfit_name()).with_extension("package");
			let mut resources = vec![DecodedResource::Gzps(outfit.gzps.clone())];
			if let Some(gzps_idr) = &outfit.gzps_idr {
				resources.push(DecodedResource::Idr(gzps_idr.clone()));
			}
			if let Some(binx) = &outfit.binx {
				resources.push(DecodedResource::Binx(binx.clone()));
			}
			if let Some(binx_idr) = &outfit.binx_idr {
				resources.push(DecodedResource::Idr(binx_idr.clone()));
			}
			Dbpf::write_package_file(&resources, &file_name)?;
		}
	}
	println!("DONE");

	Ok(())
}
