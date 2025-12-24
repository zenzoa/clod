use std::error::Error;
use std::io::{ self, Write };
use std::path::PathBuf;

use crate::dbpf::{ Dbpf, Identifier, TypeId };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Age, Gender, Category, HairTone };
use crate::outfit::Outfit;

use super::{ get_default_replacement_files, extract_resources, extract_gzps, default_output_path };

pub fn default_hair(
		source: Option<PathBuf>,
		output: Option<PathBuf>,
		add_ages: bool,
		all_categories: bool,
		visible: Option<bool>,
		townified: Option<bool>,
		hat: Option<bool>,
		hide_pack_icon: bool,
		same_family: bool
	) -> Result<(), Box<dyn Error>> {

	let source_dir = source.unwrap_or(PathBuf::from("./"));

	let output_path = output.unwrap_or(default_output_path(&source_dir, "DEFAULT"));

	let (original_files, replacement_files) = get_default_replacement_files(&source_dir)?;

	// get all GZPS resources in original package(s)
	let gzps_list = extract_gzps(&original_files)?;
	let mut pairings: Vec<Option<usize>> = gzps_list.iter().map(|_| None).collect();
	if gzps_list.is_empty() {
		return Err("No GZPS resources found for original hairs".into());
	}

	// get all resources from replacement package(s)
	let resources = extract_resources(&replacement_files)?;

	// sort replacement resources into hairs
	let mut replacement_hairs = Vec::new();
	for resource in &resources {
		if let DecodedResource::Gzps(gzps) = resource {
			let mut hair = Outfit::from_resources(gzps.clone(), &resources, true)?;
			if hair.gzps.hairtone == HairTone::None {
				hair.gzps.hairtone = HairTone::Other;
			}
			replacement_hairs.push(hair);
		}
	}
	if replacement_hairs.is_empty() {
		return Err("No replacement hairs found".into());
	}

	let first_family = &gzps_list[0].family;

	let mut gender: Option<Gender> = None;
	let mut age_color_sets = Vec::new();
	let mut separate_youngadult = false;

	let mut unreplaced_warnings = Vec::new();

	for (i, gzps) in gzps_list.iter().enumerate() {
		if gzps.genders.len() == 1 {
			gender = Some(gzps.genders[0]);
		}
		if !gzps.ages.contains(&Age::Adult) && gzps.ages.contains(&Age::YoungAdult) {
			separate_youngadult = true;
		}
		for (j, hair) in replacement_hairs.iter().enumerate() {
			if gzps.hairtone == hair.gzps.hairtone &&
				Age::are_compatible(&gzps.ages, &hair.gzps.ages) &&
				Gender::are_compatible(&gzps.genders, &hair.gzps.genders, &gzps.ages) &&
				pairings[i].is_none() {
					println!("Replacing {}", gzps.title);
					pairings[i] = Some(j);
					for age in &gzps.ages {
						age_color_sets.push(format!("{}_{}", Age::stringify(&[*age], false, false), gzps.hairtone.stringify()));
					}
			}
		}
		if pairings[i].is_none() {
			let hidden = if gzps.flags & 1 == 1 { " (HIDDEN)" } else { "" };
			unreplaced_warnings.push(format!("WARNING: \"{}\"{} not replaced", gzps.name, hidden));
		}
	}

	let mut extra_hairs = Vec::new();
	if add_ages {
		for (j, hair) in replacement_hairs.iter().enumerate() {
			if !pairings.contains(&Some(j)) && gender.is_none_or(|g| hair.gzps.genders.contains(&g)) {
				let mut ages_to_add = Vec::new();
				for age in &hair.gzps.ages {
					let age_color = format!("{}_{}", Age::stringify(&[*age], false, false), hair.gzps.hairtone.stringify());
					if !age_color_sets.contains(&age_color) && (*age != Age::YoungAdult || separate_youngadult) {
						ages_to_add.push(*age);
						age_color_sets.push(age_color.clone());
						println!("Adding {age_color}");
					}
				}
				if !ages_to_add.is_empty() {
					extra_hairs.push((j, ages_to_add));
				}
			}
		}
	}

	for warning in unreplaced_warnings {
		println!("{warning}");
	}

	// replace original hairs
	let mut new_hairs = Vec::new();
	for (i, replacement_hair_index) in pairings.iter().enumerate() {
		if let Some(j) = *replacement_hair_index {
			let mut new_gzps = gzps_list[i].clone();
			let mut new_hair = replacement_hairs[j].clone();

			// copy over overrides from replacement to original GZPS
			new_gzps.resource = new_hair.gzps.resource;
			new_gzps.shape = new_hair.gzps.shape;
			new_gzps.overrides = new_hair.gzps.overrides.clone();

			// adjust age if necessary
			if !separate_youngadult && new_gzps.ages.contains(&Age::Adult) && !new_gzps.ages.contains(&Age::YoungAdult) {
				new_gzps.ages.push(Age::YoungAdult);
			}

			// enable for all categories
			if all_categories {
				new_gzps.categories = vec![
					Category::Everyday,
					Category::Swimwear,
					Category::PJs,
					Category::Formal,
					Category::Undies,
					Category::Maternity,
					Category::Athletic,
					Category::Outerwear,
				]
			}

			// set flags
			if let Some(visible) = visible {
				if !visible && new_gzps.flags & 1 == 0 {
					new_gzps.flags += 1;
				} else if visible && new_gzps.flags & 1 > 0 {
					new_gzps.flags -= 1;
				}
			}
			if let Some(townified) = townified {
				if !townified && new_gzps.flags & 8 == 0 {
					new_gzps.flags += 8;
				} else if townified && new_gzps.flags & 8 > 0 {
					new_gzps.flags -= 8;
				}
			}
			if let Some(hat) = hat {
				if hat && new_gzps.flags & 2 == 0 {
					new_gzps.flags += 2;
				} else if !hat && new_gzps.flags & 2 > 0 {
					new_gzps.flags -= 2;
				}
			}

			// replace family with new value
			if same_family {
				new_gzps.family = first_family.clone();
			}

			// set product to Base Game to remove pack icon
			if hide_pack_icon {
				new_gzps.product = Some(1);
			}

			// update 3IDR's TGIR to match GZPS's TGIR
			new_hair.idr.id.group_id = new_gzps.id.group_id;
			new_hair.idr.id.instance_id = new_gzps.id.instance_id;
			new_hair.idr.id.resource_id = new_gzps.id.resource_id;

			if gzps_list[i].flags & 1 == 0 {
				// remove unnecessary 3IDR properties for visible hair
				new_hair.idr.ui_ref = None;
				new_hair.idr.str_ref = None;
				new_hair.idr.coll_ref = None;
				new_hair.idr.gzps_ref = None;

			} else {
				// add required references to 3IDR for hidden hair (in case it's a hidden clone)
				new_hair.idr.ui_ref = Some(Identifier::new(u32::from(TypeId::Ui), 0, 0, 0));
				new_hair.idr.str_ref = Some(Identifier::new(u32::from(TypeId::TextList), 0x7F43F357, 0, 1));
				new_hair.idr.coll_ref = Some(Identifier::new(u32::from(TypeId::Coll), 0x7F43F357, 0, 0x6CDBC43D));
				new_hair.idr.gzps_ref = Some(new_gzps.id.clone());

				// create BINX resource
				new_hair.generate_binx();
			}

			// copy new GZPS back to outfit
			new_hair.gzps = new_gzps;

			// add outfit to list
			new_hairs.push(new_hair);
		}
	}

	// add extra hairs
	new_hairs.extend_from_slice(&extra_hairs.iter().map(|(hair_index, ages)| {
		let mut new_hair = replacement_hairs[*hair_index].clone();
		let gzps = &new_hairs[0].gzps;

		new_hair.gzps.ages = ages.clone();

		// copy relevant values from GZPS updated in the last step
		new_hair.gzps.version = gzps.version;
		new_hair.gzps.product = gzps.product;
		new_hair.gzps.creator = gzps.creator.clone();
		new_hair.gzps.family = gzps.family.clone();
		new_hair.gzps.flags = gzps.flags;
		new_hair.gzps.categories = gzps.categories.clone();
		new_hair.gzps.skintone = gzps.skintone.clone();

		new_hair.gzps.genetic = Some(0.0);
		new_hair.gzps.priority = None;

		// add required references to 3IDR
		new_hair.idr.ui_ref = Some(Identifier::new(u32::from(TypeId::Ui), 0, 0, 0));
		new_hair.idr.str_ref = Some(Identifier::new(u32::from(TypeId::TextList), 0x7F43F357, 0, 1));
		new_hair.idr.coll_ref = Some(Identifier::new(u32::from(TypeId::Coll), 0x7F43F357, 0, 0x6CDBC43D));
		new_hair.idr.gzps_ref = Some(new_hair.gzps.id.clone());

		// create BINX resource
		new_hair.generate_binx();

		new_hair
	}).collect::<Vec<Outfit>>());

	// pull all resources together
	let all_resources = new_hairs
		.iter()
		.flat_map(|o| o.get_resources())
		.collect::<Vec<DecodedResource>>();

	// save package file
	print!("Saving and compressing package...");
	io::stdout().flush()?;
	Dbpf::write_package_file(&all_resources, &output_path, true)?;
	println!(" DONE");

	Ok(())
}
