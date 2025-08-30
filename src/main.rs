use std::error::Error;
use std::env::args;
use std::fs;
use std::fs::{ File, DirEntry };
use std::io::{ Cursor, Write };
use std::path::Path;

use cursive::With;
use cursive::view::Scrollable;
use cursive::views::{ Dialog, ListView, SelectView };

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Age, Gender };
use crate::outfit::Outfit;

mod dbpf;
mod outfit;

#[derive(Clone, Default)]
struct SivData {
	output_path: String,
	gzps_list: Vec<Gzps>,
	outfits: Vec<Outfit>,
	pairings: Vec<Option<usize>>
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
	let original_path = args().nth(1).expect("No file passed in as argument.");
	let replacement_path = args().nth(2).expect("No replacement folder passed in as argument.");

	let original_filename = Path::new(&original_path).file_name().unwrap().to_string_lossy();

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
			let outfit = Outfit::from_resources(gzps.clone(), &resources)?;
			outfits.push(outfit);
		}
	}

	// get all GZPS resources in original package
	let bytes = fs::read(&original_path)?;
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

	//
	// let mut cur = Cursor::new(Vec::new());
	// original_dbpf.write(&mut cur)?;
	// let mut new_file = File::create(&original_path.replace(".package", "_DEFAULT.package"))?;
	// new_file.write(&cur.into_inner())?;
	//

	let mut siv = cursive::default();

	let data = SivData {
		output_path: original_path.replace(".package", "_DEFAULT.package"),
		gzps_list,
		outfits,
		pairings
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

	for (i, outfit_index) in data.pairings.iter().enumerate() {
		if let Some(j) = *outfit_index {
			let mut new_gzps = data.gzps_list[i].clone();
			let mut new_outfit = data.outfits[j].clone();

			// copy over shoe/overrides from replacement to original GZPS
			new_gzps.shoe = new_outfit.gzps.shoe;
			new_gzps.overrides = new_outfit.gzps.overrides.clone();

			// enable for both genders if baby/toddler/child, and enabled for young adult + adult
			if new_gzps.age.contains(&Age::Baby) || new_gzps.age.contains(&Age::Toddler) || new_gzps.age.contains(&Age::Child) {
				new_gzps.gender = vec![Gender::Male, Gender::Female];
			} else if new_gzps.age.contains(&Age::YoungAdult) && !new_gzps.age.contains(&Age::Adult) {
				new_gzps.age.push(Age::Adult);
			} else if new_gzps.age.contains(&Age::Adult) && !new_gzps.age.contains(&Age::YoungAdult) {
				new_gzps.age.push(Age::YoungAdult);
			}

			// update idr's tgir to match gzps's tgir
			new_outfit.idr.id.group_id = new_gzps.id.group_id;
			new_outfit.idr.id.instance_id = new_gzps.id.instance_id;
			new_outfit.idr.id.resource_id = new_gzps.id.resource_id;

			// copy new gzps back to outfit
			new_outfit.gzps = new_gzps;

			new_outfits.push(new_outfit);
		}
	}

	// save replaced outfits as new file (original path with ".package" replaced with "_DEFAULT.package")
	let resources = new_outfits.iter().flat_map(|o| o.get_resources()).collect::<Vec<DecodedResource>>();
	let mut new_dbpf = Dbpf::new(resources)?;
	new_dbpf.clean_up_resources();

	let mut cur = Cursor::new(Vec::new());
	new_dbpf.write(&mut cur)?;

	let mut new_file = File::create(&data.output_path)?;
	new_file.write_all(&cur.into_inner())?;

	Ok(())
}
