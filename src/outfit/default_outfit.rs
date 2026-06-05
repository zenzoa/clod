use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use crate::dbpf::{ Dbpf, Identifier };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::text_list::TextList;
use crate::dbpf::resource_types::gzps::{ Age, Gender };

use crate::helpers::{ ExtractedOutfit, ReplacementOutfit, get_folder_name, get_outfit_related_resources, get_gzps_related_resources, get_packages_in_dir, get_resources_in_packages, get_subfolders, read_properties_file };

pub fn default_outfit(original: PathBuf, replacement: Option<PathBuf>, output: Option<PathBuf>, include_extras: bool, quiet: bool) -> Result<(), Box<dyn Error>> {
	let mut sort = 0;

	if !quiet { print!("Reading original outfit template(s)..."); }
	let original_packages = get_packages_in_dir(&original)?;
	let original_resources = get_resources_in_packages(&original_packages)?;
	let mut original_outfits = Vec::new();
	for original_resource in &original_resources {
		if let DecodedResource::Gzps(gzps) = original_resource {
			let (gzps_idr, binx_idr, binx) = get_gzps_related_resources(&gzps.id, &original_resources, &original_resources);
			if let Some(binx) = &binx {
				if binx.sort != 0 {
					sort = binx.sort
				}
			}
			original_outfits.push(ExtractedOutfit {
				gzps: gzps.clone(),
				gzps_idr,
				binx_idr,
				binx
			})
		}
	}
	original_outfits.sort_by_key(|o| o.gzps.name.to_string());
	if !quiet { println!("DONE"); }

	let mut replacement_outfits = Vec::new();

	let replacement = replacement.unwrap_or_else(|| {
		get_subfolders(&original)
		.map(|subfolders| subfolders.first().cloned())
		.unwrap_or(None)
		.unwrap_or_default()
	});
	if !replacement.exists() {
		return Err("Replacement folder does not exist".into());
	}

	if !quiet { print!("Reading replacement outfit packages..."); }
	let replacement_packages = get_packages_in_dir(&replacement)?;
	let replacement_resources = get_resources_in_packages(&replacement_packages)?;
	for replacement_resource in &replacement_resources {
		if let DecodedResource::Gzps(gzps) = replacement_resource {
			let (gzps_idr, binx_idr, binx) = get_gzps_related_resources(&gzps.id, &replacement_resources, &replacement_resources);
			let mut title = String::new();
			let mut cres_id = None;
			let mut shpe_id = None;
			let mut txmt_ids = None;

			if let Some(gzps_idr) = gzps_idr {
				cres_id = gzps_idr.references.get(gzps.cres_index as usize).cloned();
				shpe_id = gzps_idr.references.get(gzps.shpe_index as usize).cloned();
				let mut txmt_ids_inner = Vec::new();
				for subset_index in &gzps.subset_indexes {
					if let Some(txmt_id) = gzps_idr.references.get(subset_index.txmt_index as usize) {
						txmt_ids_inner.push((subset_index.subset_name.to_string(), txmt_id.clone()));
					}
				}
				if txmt_ids_inner.len() == gzps.subset_indexes.len() {
					txmt_ids = Some(txmt_ids_inner);
				}
			}

			if let (Some(binx), Some(binx_idr)) = (binx, binx_idr) {
				if let Some(DecodedResource::TextList(text_list)) = binx_idr.references
					.get(binx.text_list_index as usize)
					.map(|text_list_ref| replacement_resources.iter()
						.find(|r| r.get_id() == *text_list_ref))
					.unwrap_or(None) {
						if let Some(text) = text_list.strings.first() {
							title = text.title.clone();
						}
					}
			}

			if let (Some(cres_id), Some(shpe_id), Some(txmt_ids)) = (cres_id, shpe_id, txmt_ids) {
				replacement_outfits.push(ReplacementOutfit {
					title,
					gzps: gzps.clone(),
					cres_id,
					shpe_id,
					txmt_ids: txmt_ids.clone()
				});
			}
		}
	}
	replacement_outfits.sort_by_key(|o| o.title.clone());
	if !quiet { println!("DONE"); }

	let (category_overrides, flag_overrides, make_unisex) = read_properties_file(&original, quiet)?;

	if !quiet { println!("Making replacements..."); }
	let mut used_replacement_outfits = Vec::new();
	let mut used_resources: HashMap<Identifier, DecodedResource> = HashMap::new();
	for original_outfit in original_outfits.iter_mut() {
		let mut is_replaced = false;
		for (i, replacement_outfit) in replacement_outfits.iter().enumerate() {
			if Age::are_compatible(&original_outfit.gzps.ages, &replacement_outfit.gzps.ages) &&
				Gender::are_compatible(&original_outfit.gzps.genders, &replacement_outfit.gzps.genders, &replacement_outfit.gzps.ages) &&
				!original_outfit.gzps.parts.is_empty() &&
				!replacement_outfit.gzps.parts.is_empty() &&
				original_outfit.gzps.parts[0] == replacement_outfit.gzps.parts[0] &&
				!used_replacement_outfits.contains(&i) {
					used_replacement_outfits.push(i);

					// Update GZPS
					original_outfit.gzps.ages = replacement_outfit.gzps.ages.clone();
					original_outfit.gzps.genders = replacement_outfit.gzps.genders.clone();
					original_outfit.gzps.update_with(make_unisex, &category_overrides, &flag_overrides, &replacement_outfit.txmt_ids);

					// Get resources used by outfit
					used_resources.extend(get_outfit_related_resources(
						&original_outfit.gzps,
						&original_outfit.binx,
						&original_outfit.binx_idr,
						replacement_outfit,
						&replacement_resources,
						sort,
						true
					));

					if !quiet { println!("  {}: replaced with \"{}\"", original_outfit.gzps.name, replacement_outfit.title); }
					is_replaced = true;
					break;
			}
		}
		if !is_replaced {
			println!("  {}: NO REPLACEMENT FOUND", original_outfit.gzps.name);
		}
	}
	if !quiet { println!("DONE"); }

	let output_path = output.unwrap_or_else(||
		get_folder_name(&original)
		.map(|base| original.join(format!("{base}_DEFAULT.package")))
		.unwrap_or_default()
	);
	if output_path.to_string_lossy() == "" {
		return Err("Invalid output path".into())
	}

	if include_extras && replacement_outfits.len() > used_replacement_outfits.len() {
		if !quiet { println!("Finding EXTRAS..."); }
		let mut extra_resources: HashMap<Identifier, DecodedResource> = HashMap::new();
		for (i, extra_outfit) in replacement_outfits.iter_mut().enumerate() {
			if !used_replacement_outfits.contains(&i) {
				// Update GZPS to match defaults
				extra_outfit.gzps.update_with(make_unisex, &category_overrides, &flag_overrides, &extra_outfit.txmt_ids);

				// Create text list
				let text_list = TextList::from_string(&extra_outfit.title, extra_outfit.gzps.id.group_id);
				extra_resources.insert(text_list.id.clone(), DecodedResource::TextList(text_list));

				// Get resources used by outfit
				let outfit_resources = get_outfit_related_resources(
					&extra_outfit.gzps,
					&None,
					&None,
					extra_outfit,
					&replacement_resources,
					sort,
					true
				);
				for (id, resource) in outfit_resources {
					if !used_resources.contains_key(&id) {
						extra_resources.insert(id, resource);
					}
				}

				if !quiet { println!("  added \"{}\"", extra_outfit.title); }
			}
		}

		if !extra_resources.is_empty() {
			let extras_file_name = output_path.file_stem().map(|stem| {
				let stem = stem.to_string_lossy().to_string();
				if stem.contains("DEFAULT") {
					stem.replace("DEFAULT", "EXTRAS")
				} else {
					format!("{stem}_EXTRAS")
				}
			}).unwrap_or("EXTRAS".to_string());
			let extras_path = output_path.with_file_name(extras_file_name).with_extension("package");

			let mut extra_resources = extra_resources.into_values().collect::<Vec<DecodedResource>>();
			extra_resources.sort_by_key(|r| r.get_id().sort_key());
			Dbpf::write_package_file(&extra_resources, &extras_path)?;
		}
		if !quiet { println!("DONE"); }
	}

	if !quiet { if !quiet { print!("Writing DEFAULT package..."); } }
	let mut resources = used_resources.into_values().collect::<Vec<DecodedResource>>();
	resources.sort_by_key(|r| r.get_id().sort_key());
	Dbpf::write_package_file(&resources, &output_path)?;
	if !quiet { println!("DONE"); }

	Ok(())
}
