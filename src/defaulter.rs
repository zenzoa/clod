use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;

use cursive::With;
use cursive::view::Scrollable;
use cursive::views::{ Dialog, ListView, SelectView };

use crate::dbpf::{ Dbpf, Identifier, TypeId };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Age, Gender };
use crate::dbpf::resource_types::text_list::TextList;
use crate::outfit::Outfit;

#[derive(Clone, Default)]
pub struct DefaultSettings {
	pub compress: bool,
	pub ignore_missing: bool,
	pub gender_fix: bool,
	pub product_fix: bool,
	pub flag_fix: bool
}

#[derive(Clone, Default)]
struct SivData {
	output_path: String,
	gzps_list: Vec<Gzps>,
	outfits: Vec<Outfit>,
	pairings: Vec<Option<usize>>,
	settings: DefaultSettings
}

pub fn default_outfit(original_path: &Path, replacement_path: &Path, settings: DefaultSettings) -> Result<(), Box<dyn Error>> {
	let original_filename = original_path.file_name().unwrap().to_string_lossy();

	// read all packages in replacement folder
	let mut resources = Vec::new();
	let mut dir_entries: Vec<DirEntry> = fs::read_dir(replacement_path)?
		.filter_map(|entry|
			match entry {
				Ok(entry) => Some(entry),
				Err(_) => None
			}).collect();
	dir_entries.sort_by_key(|entry| entry.file_name().to_string_lossy().into_owned());
	for dir_entry in dir_entries {
		if let Ok(filename) = dir_entry.file_name().into_string() {
			if filename.ends_with(".package") && filename != original_filename {
				let bytes = fs::read(dir_entry.path())?;
				let dbpf = Dbpf::read(&bytes, &filename.replace(".package", ""))?;
				resources.extend_from_slice(&dbpf.resources);
			}
		}
	}

	// sort replacement resources into outfits
	let mut outfits = Vec::new();
	for resource in &resources {
		if let DecodedResource::Gzps(gzps) = resource {
			let outfit = Outfit::from_resources(gzps.clone(), &resources, settings.ignore_missing)?;
			outfits.push(outfit);
		}
	}

	// get all GZPS resources in original package
	let bytes = fs::read(original_path)?;
	let original_dbpf = Dbpf::read(&bytes, "")?;
	let mut gzps_list: Vec<Gzps> = original_dbpf.resources
		.iter()
		.filter_map(|res|
			if let DecodedResource::Gzps(gzps) = res {
				Some(gzps.clone())
			} else {
				None
			}
		).collect();
	gzps_list.sort_by_key(|gzps| gzps.name.to_string());
	let pairings: Vec<Option<usize>> = gzps_list.iter().map(|_| None).collect();

	let mut siv = cursive::default();

	let data = SivData {
		output_path: original_path.to_str().ok_or("Unable to convert path to string")?.replace(".package", "_DEFAULT.package"),
		gzps_list,
		outfits,
		pairings,
		settings
	};
	siv.set_user_data(data.clone());

	siv.add_layer(
		Dialog::around(
			ListView::new().with(|list| {
				for (i, gzps) in data.gzps_list.iter().enumerate() {
					list.add_child(gzps.name.to_string(), SelectView::new()
						.with(|select| {
							select.add_item("-", (i, None));
							for (j, outfit) in data.outfits.iter().enumerate() {
								if Age::are_compatible(&gzps.age, &outfit.gzps.age) &&
									Gender::are_compatible(&gzps.gender, &outfit.gzps.gender, &gzps.age) {
										select.add_item(&outfit.title, (i, Some(j)));
								}
							}
						})
						.on_submit(|s, item| {
							s.with_user_data(|user_data: &mut SivData| {
								user_data.pairings[item.0] = item.1;
							});
						})
						.popup().scrollable())
				}
			}).scrollable())
			.title(original_filename.replace(".package", ""))
			.button("Quit", |s| {
				s.quit();
			})
			.button("Save", |s| {
				s.with_user_data(|user_data: &mut SivData| {
					let _ = save_package(user_data);
				});
				s.quit();
			})
	);

	siv.try_run()?;

	Ok(())
}

fn save_package(data: &SivData) -> Result<(), Box<dyn Error>> {
	let mut new_outfits = Vec::new();

	let mut text_lists: Vec<TextList> = Vec::new();

	for (i, outfit_index) in data.pairings.iter().enumerate() {
		if let Some(j) = *outfit_index {
			let mut new_gzps = data.gzps_list[i].clone();
			let mut new_outfit = data.outfits[j].clone();

			// copy over shoe/overrides from replacement to original GZPS
			new_gzps.shoe = new_outfit.gzps.shoe;
			new_gzps.overrides = new_outfit.gzps.overrides.clone();

			// enable for both genders if baby/toddler/child
			if data.settings.gender_fix && (new_gzps.age.contains(&Age::Baby) || new_gzps.age.contains(&Age::Toddler) || new_gzps.age.contains(&Age::Child)) {
				new_gzps.gender = vec![Gender::Male, Gender::Female];
			}

			// enable for young adult + adult
			if new_gzps.age.contains(&Age::YoungAdult) && !new_gzps.age.contains(&Age::Adult) {
				new_gzps.age.push(Age::Adult);
			} else if new_gzps.age.contains(&Age::Adult) && !new_gzps.age.contains(&Age::YoungAdult) {
				new_gzps.age.push(Age::YoungAdult);
			}

			// set product to Base Game to remove pack icon
			if data.settings.product_fix {
				new_gzps.product = Some(1);
			}

			// update 3IDR's TGIR to match GZPS's TGIR
			new_outfit.idr.id.group_id = new_gzps.id.group_id;
			new_outfit.idr.id.instance_id = new_gzps.id.instance_id;
			new_outfit.idr.id.resource_id = new_gzps.id.resource_id;

			// make young adult clones visible in catalog
			let outfit_name = new_gzps.name.to_string().to_lowercase();
			if outfit_name.starts_with('y') && outfit_name.contains("clone") && new_gzps.flags == 9 {
				// create a STR# if none exists with this outfit's group id
				let text_list_id = Identifier::new(TypeId::TextList as u32, new_gzps.id.group_id, 0x1, 0);
				if text_lists.iter().find(|t| t.id.group_id == new_gzps.id.group_id).is_none() {
					text_lists.push(TextList::create_empty(text_list_id.clone()));
				}

				// create BINX resource
				new_outfit.binx = Some(new_outfit.generate_binx());

				// add additional references to 3IDR
				new_outfit.idr.ui_ref = Some(Identifier::new(TypeId::Ui as u32, 0, 0, 0));
				new_outfit.idr.str_ref = Some(text_list_id.clone());
				new_outfit.idr.coll_ref = Some(Identifier::new(TypeId::Coll as u32, 0x0FFEFEFE, 0x0FFE0080, 0));
				new_outfit.idr.gzps_ref = Some(new_gzps.id.clone());

				// set flags so outfit won't be hidden
				if new_gzps.flags & 1 == 1 {
					new_gzps.flags -= 1;
				}

			} else {
				// if not adding BINX, remove unnecessary 3IDR properties
				new_outfit.idr.ui_ref = None;
				new_outfit.idr.str_ref = None;
				new_outfit.idr.coll_ref = None;
				new_outfit.idr.gzps_ref = None;
			}

			// set flags to 0 to make it visible and townified
			if data.settings.flag_fix {
				new_gzps.flags = 0
			}

			// copy new GZPS back to outfit
			new_outfit.gzps = new_gzps;

			// add outfit to list
			new_outfits.push(new_outfit);
		}
	}

	// pull all resources together
	let mut resources = new_outfits
		.iter()
		.flat_map(|o| o.get_resources())
		.collect::<Vec<DecodedResource>>();

	// add any STR# resources that were made
	let text_list_resources = text_lists
		.iter()
		.map(|t| DecodedResource::TextList(t.clone()))
		.collect::<Vec<DecodedResource>>();
	resources.extend_from_slice(&text_list_resources);

	// save package file
	Dbpf::write_package_file(&resources, &data.output_path, data.settings.compress)?;

	Ok(())
}
