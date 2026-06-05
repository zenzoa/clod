use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use crate::dbpf::{ Dbpf, Identifier, PascalString, TypeId };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::text_list::TextList;
use crate::dbpf::resource_types::gzps::{ Age, Category, Gender, OutfitFlag };

use crate::helpers::{ ExtractedOutfit, ReplacementOutfit, get_folder_name, get_gzps_related_resources, get_outfit_related_resources, get_packages_in_dir, get_resources_in_packages, get_subfolders, read_properties_file };

pub fn default_hair(original: PathBuf, replacement: Option<PathBuf>, fallback: Option<PathBuf>, output: Option<PathBuf>, include_extras: bool, quiet: bool) -> Result<(), Box<dyn Error>> {
	let mut sort = 0;
	let mut family = None;

	let mut has_standalone_adult = false;
	let mut has_standalone_youngadult = false;

	let mut is_hat = false;
	let mut hat_ages = Vec::new();

	if !quiet { print!("Reading original hair templates..."); }
	let original_packages = get_packages_in_dir(&original)?;
	let original_resources = get_resources_in_packages(&original_packages)?;
	let mut original_hairs = Vec::new();
	for original_resource in &original_resources {
		if let DecodedResource::Gzps(gzps) = original_resource {
			let (gzps_idr, binx_idr, binx) = get_gzps_related_resources(&gzps.id, &original_resources, &original_resources);
			if let Some(binx) = &binx {
				if binx.sort != 0 {
					sort = binx.sort
				}
			}
			if family.is_none() {
				family = Some(gzps.family.clone());
			}
			if gzps.ages.contains(&Age::Adult) && !gzps.ages.contains(&Age::YoungAdult) && binx.is_some() {
				has_standalone_adult = true;
			}
			if gzps.ages.contains(&Age::YoungAdult) && !gzps.ages.contains(&Age::Adult) && binx.is_some() {
				has_standalone_youngadult = true;
			}
			if gzps.flags.contains(&OutfitFlag::Hat) {
				is_hat = true;
				for age in &gzps.ages {
					if !hat_ages.contains(age) {
						hat_ages.push(*age);
					}
				}
			}
			original_hairs.push(ExtractedOutfit {
				gzps: gzps.clone(),
				gzps_idr,
				binx_idr,
				binx
			})
		}
	}
	original_hairs.sort_by_key(|o| o.gzps.name.to_string());
	if !quiet { println!("DONE"); }

	let has_separate_adult_and_youngadult = has_standalone_adult && has_standalone_youngadult;

	let replacement = replacement.unwrap_or_else(|| {
		get_subfolders(&original)
		.map(|subfolders| subfolders.first().cloned())
		.unwrap_or(None)
		.unwrap_or_default()
	});
	if !replacement.exists() {
		return Err("Replacement folder does not exist".into());
	}

	if !quiet { print!("Reading replacement hair packages..."); }
	let replacement_packages = get_packages_in_dir(&replacement)?;
	let replacement_resources = get_resources_in_packages(&replacement_packages)?;
	let mut replacement_hairs = Vec::new();
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
				replacement_hairs.push(ReplacementOutfit {
					title,
					gzps: gzps.clone(),
					cres_id,
					shpe_id,
					txmt_ids: txmt_ids.clone()
				});
			}
		}
	}
	replacement_hairs.sort_by_key(|o| o.title.clone());
	if !quiet { println!("DONE"); }

	let mut fallback_hairs = Vec::new();
	if let Some(fallback) = fallback {
		if !quiet { print!("Reading fallback hair packages..."); }
		let fallback_packages = get_packages_in_dir(&fallback)?;
		let fallback_resources = get_resources_in_packages(&fallback_packages)?;
		for fallback_resource in &fallback_resources {
			if let DecodedResource::Gzps(gzps) = fallback_resource {
				let (gzps_idr, binx_idr, binx) = get_gzps_related_resources(&gzps.id, &fallback_resources, &fallback_resources);
				fallback_hairs.push(ExtractedOutfit {
					gzps: gzps.clone(),
					gzps_idr,
					binx_idr,
					binx
				})
			}
		}
		if !quiet { println!("DONE"); }
	}

	let (mut category_overrides, mut flag_overrides, mut make_unisex) = read_properties_file(&original, quiet)?;
	if category_overrides.is_none() {
		category_overrides = Some(Category::all());
		flag_overrides = Some(vec![]);
		make_unisex = true;
	}

	let mut adult_added = false;
	let mut unisex_toddler_added = false;
	let mut unisex_child_added = false;

	if !quiet { println!("Making replacements..."); }
	let mut used_replacement_hairs = Vec::new();
	let mut used_resources: HashMap<Identifier, DecodedResource> = HashMap::new();
	for original_hair in original_hairs.iter_mut() {
		let mut is_replaced = false;
		for (i, replacement_hair) in replacement_hairs.iter().enumerate() {
			let mut is_hat_off_hair = false;
			for age in &original_hair.gzps.ages {
				if hat_ages.contains(age) {
					is_hat_off_hair = true;
				}
			}
			if ((original_hair.gzps.ages.len() == 1 && replacement_hair.gzps.ages.len() == 1 && original_hair.gzps.ages[0] == replacement_hair.gzps.ages[0]) ||
					(original_hair.gzps.ages.contains(&Age::Adult) && replacement_hair.gzps.ages.contains(&Age::Adult))) &&
				original_hair.gzps.genders[0] == replacement_hair.gzps.genders[0] &&
				original_hair.gzps.hairtone == replacement_hair.gzps.hairtone &&
				(!is_hat || original_hair.gzps.flags.contains(&OutfitFlag::Hat) || !is_hat_off_hair) &&
				!used_replacement_hairs.contains(&i) {
					used_replacement_hairs.push(i);

					// Check if this is a young adult clone, and should stay hidden
					let youngadult_clone = original_hair.gzps.ages.contains(&Age::YoungAdult) && !original_hair.gzps.ages.contains(&Age::Adult) && original_hair.binx.is_none() && original_hair.binx_idr.is_none();

					// Update GZPS
					original_hair.gzps.update_with(make_unisex, &category_overrides, &flag_overrides, &replacement_hair.txmt_ids);
					if (original_hair.gzps.ages.contains(&Age::Adult) || original_hair.gzps.ages.contains(&Age::YoungAdult)) && !youngadult_clone && !has_separate_adult_and_youngadult {
						original_hair.gzps.ages = vec![Age::YoungAdult, Age::Adult];
					}
					if youngadult_clone && !original_hair.gzps.flags.contains(&OutfitFlag::Hidden) {
						original_hair.gzps.flags.push(OutfitFlag::Hidden)
					}
					if youngadult_clone && !original_hair.gzps.flags.contains(&OutfitFlag::NoTownies) {
						original_hair.gzps.flags.push(OutfitFlag::NoTownies)
					}

					// Track certain ages to make sure they're not duplicated when creating extras
					if original_hair.gzps.ages.contains(&Age::Adult) {
						adult_added = true;
					} else if make_unisex && original_hair.gzps.ages.contains(&Age::Toddler) {
						unisex_toddler_added = true;
					} else if make_unisex && original_hair.gzps.ages.contains(&Age::Child) {
						unisex_child_added = true;
					}

					// Make sure all the replacements have the same family
					if let Some(family) = &family {
						original_hair.gzps.family = family.clone();
					}

					// Get resources used by hair
					let create_binx = original_hair.binx.is_none() && original_hair.binx_idr.is_none() && !youngadult_clone;
					used_resources.extend(get_outfit_related_resources(
						&original_hair.gzps,
						&original_hair.binx,
						&original_hair.binx_idr,
						replacement_hair,
						&replacement_resources,
						sort,
						create_binx
					));

					let binx_string = if create_binx { ", added BINX" } else { "" };
					let clone_string = if youngadult_clone { ", ya clone" } else { "" };
					if !quiet { println!("  {}: replaced with \"{}\" ({}{}{})", original_hair.gzps.name, replacement_hair.title, original_hair.gzps.age_gender_string(), binx_string, clone_string); }
					is_replaced = true;
					break;
			}
		}

		if !is_replaced && original_hair.binx.is_none() {
			for fallback_hair in &fallback_hairs {
				if Age::are_compatible(&original_hair.gzps.ages, &fallback_hair.gzps.ages) &&
					Gender::are_compatible(&original_hair.gzps.genders, &fallback_hair.gzps.genders, &original_hair.gzps.ages) &&
					original_hair.gzps.hairtone == fallback_hair.gzps.hairtone {
						if let Some(fallback_idr) = &fallback_hair.gzps_idr {
							// Make sure all the replacements have the same family
							if let Some(family) = &family {
								original_hair.gzps.family = family.clone();
							}

							// Update gzps indexes
							original_hair.gzps.shpe_index = fallback_hair.gzps.shpe_index;
							original_hair.gzps.cres_index = fallback_hair.gzps.cres_index;
							original_hair.gzps.subset_indexes = fallback_hair.gzps.subset_indexes.clone();
							used_resources.insert(original_hair.gzps.id.clone(), DecodedResource::Gzps(original_hair.gzps.clone()));

							// Update idr
							let mut original_idr = fallback_idr.clone();
							original_idr.id = original_hair.gzps.id.with_type_id(TypeId::Idr);
							used_resources.insert(original_idr.id.clone(), DecodedResource::Idr(original_idr));

							if !quiet { println!("  {}: used fallback", original_hair.gzps.name); }
							is_replaced = true;
							break;
						}
					}
			}
		}

		if !is_replaced && !quiet {
			// change family so that the fallback hair will be the regular default (and pick up whatever replacement you have for it)
			original_hair.gzps.family = PascalString::new("0cbac043-8cdc-4e92-93f2-f3efe470f8f6");
			used_resources.insert(original_hair.gzps.id.clone(), DecodedResource::Gzps(original_hair.gzps.clone()));
			println!("  {}: NO REPLACEMENT FOUND", original_hair.gzps.name);
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

	if include_extras && replacement_hairs.len() > used_replacement_hairs.len() {
		if !quiet { println!("Finding EXTRAS..."); }
		let mut extra_resources: HashMap<Identifier, DecodedResource> = HashMap::new();
		for (i, extra_hair) in replacement_hairs.iter_mut().enumerate() {
			let same_gender = extra_hair.gzps.genders[0] == original_hairs[0].gzps.genders[0];
			if !(same_gender && adult_added && (extra_hair.gzps.ages.contains(&Age::Adult) || extra_hair.gzps.ages.contains(&Age::YoungAdult))) &&
				!(unisex_child_added && extra_hair.gzps.ages.contains(&Age::Child)) &&
				!(unisex_toddler_added && extra_hair.gzps.ages.contains(&Age::Toddler)) &&
				!used_replacement_hairs.contains(&i) {
					// Update GZPS to match defaults
					extra_hair.gzps.update_with(make_unisex, &category_overrides, &flag_overrides, &extra_hair.txmt_ids);
					if let Some(family) = &family {
						extra_hair.gzps.family = family.clone();
					}

					// Create text list
					let text_list = TextList::from_string(&extra_hair.title, extra_hair.gzps.id.group_id);
					extra_resources.insert(text_list.id.clone(), DecodedResource::TextList(text_list));

					// Get resources used by outfit
					let outfit_resources = get_outfit_related_resources(
						&extra_hair.gzps,
						&None,
						&None,
						extra_hair,
						&replacement_resources,
						sort,
						true
					);
					for (id, resource) in outfit_resources {
						if !used_resources.contains_key(&id) {
							extra_resources.insert(id, resource);
						}
					}

					if !quiet { println!("  added \"{}\" ({})", extra_hair.title, extra_hair.gzps.age_gender_string()); }
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
