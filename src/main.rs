use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::path::{ Path, PathBuf };
use std::collections::HashMap;

use clap::{ Parser, Subcommand };

use cursive::With;
use cursive::view::Scrollable;
use cursive::views::{ Dialog, ListView, SelectView };

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Age, Gender, Part };
use crate::dbpf::resource_types::idr::Idr;
use crate::outfit::Outfit;

mod dbpf;
mod outfit;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
	#[command(subcommand)]
	command: Option<Command>
}

#[derive(Subcommand)]
enum Command {
	/// Generates a default replacement for a TS2 outfit
	DefaultOutfit {
		/// Package file containing original outfit(s)' GZPS and 3IDR resources
		#[arg(short, long, value_name="FILE")]
		original: PathBuf,
		/// Folder containing replacement mesh and recolor package files
		#[arg(short, long, value_name="FOLDER")]
		replacements: Option<PathBuf>,
	},
	/// Extracts outfits from game files for use in default replacements
	ExtractOutfits {
		/// Folder containing one or more Skin.package files
		#[arg(short, long, value_name="FOLDER")]
		input: Option<PathBuf>,
		/// Folder to save extracted outfit files to
		#[arg(short, long, value_name="FOLDER")]
		output: Option<PathBuf>,
	}
}

#[derive(Clone, Default)]
struct SivData {
	output_path: String,
	gzps_list: Vec<Gzps>,
	outfits: Vec<Outfit>,
	pairings: Vec<Option<usize>>
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
	let args = Args::parse();
	match args.command {
		Some(Command::DefaultOutfit{original, replacements}) => {
			default_outfit(
				&original,
				&replacements.unwrap_or(PathBuf::from("./"))
			)
		}
		Some(Command::ExtractOutfits{input, output}) => {
			save_skins(
				&input.unwrap_or(PathBuf::from("./")),
				&output.unwrap_or(PathBuf::from("./output"))
			)
		}
		None => Err("No command given.".into())
	}
}

fn default_outfit(original_path: &Path, replacement_path: &Path) -> Result<(), Box<dyn Error>> {
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

	let mut siv = cursive::default();

	let data = SivData {
		output_path: original_path.to_str().ok_or("Unable to convert path to string")?.replace(".package", "_DEFAULT.package"),
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

fn save_skins(path: &Path, new_path: &Path) -> Result<(), Box<dyn Error>> {
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

	for (name, (gzps, idr)) in &outfits {
		if !(gzps.species == 1 &&
			gzps.parts.len() == 1 &&
			(gzps.parts[0] == Part::Body || gzps.parts[0] == Part::Top || gzps.parts[0] == Part::Bottom)) {
				continue
		}

		let name_without_prefix = name.trim_start_matches("CASIE_");

		let name_without_suffix = match name_without_prefix.split_once("_") {
			Some((first, _)) => first,
			None => name_without_prefix
		};

		let (age_gender, full_outfit_name) = name_without_suffix.split_at(2);

		let (outfit_type, outfit_name) = if full_outfit_name.starts_with("body") {
			("body", full_outfit_name.trim_start_matches("body"))
		} else if full_outfit_name.starts_with("top") {
			("top", full_outfit_name.trim_start_matches("top"))
		} else if full_outfit_name.starts_with("bottom") {
			("bottom", full_outfit_name.trim_start_matches("bottom"))
		} else {
			continue
		};

		let group_name = format!("{}_{}_{}", outfit_name, outfit_type, age_gender);

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

	for (name, resources) in &outfit_groups {
		let mut file_path = new_path.to_path_buf();
		file_path.push(format!("{name}.package"));
		Dbpf::write_package_file(resources, &file_path.to_string_lossy())?;
	}

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

	let resources = new_outfits.iter().flat_map(|o| o.get_resources()).collect::<Vec<DecodedResource>>();
	Dbpf::write_package_file(&resources, &data.output_path)?;

	Ok(())
}
