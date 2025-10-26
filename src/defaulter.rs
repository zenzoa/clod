use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::ffi::OsString;

use cursive::{ Cursive, With };
use cursive::view::{ Nameable, Scrollable, Resizable };
use cursive::views::{ Dialog, Button, TextView, EditView, Checkbox, SelectView, LinearLayout, Panel, PaddedView };

use crate::dbpf::{ Dbpf, Identifier, TypeId };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Age, Gender, Category };
use crate::dbpf::resource_types::text_list::TextList;
use crate::outfit::Outfit;

#[derive(Clone, Default)]
struct SivData {
	output_path: PathBuf,
	gzps_list: Vec<Gzps>,
	outfits: Vec<Outfit>,
	pairings: Vec<Option<usize>>
}

pub fn default_outfit(source: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
	let source_dir = source.unwrap_or(PathBuf::from("./"));

	// get original files from source dir, and replacement files from any subfolders
	let mut original_files = Vec::new();
	let mut replacement_files = Vec::new();
	for entry in fs::read_dir(&source_dir)? {
		if let Ok(entry) = entry {
			let entry_path = entry.path();
			if entry_path.is_file() && entry_path.extension().unwrap_or(&OsString::new()) == OsString::from("package") {
				original_files.push(entry_path);
			} else if entry_path.is_dir() {
				for subentry in fs::read_dir(entry_path)? {
					if let Ok(subentry) = subentry {
						let subentry_path = subentry.path();
						if subentry_path.is_file() && subentry_path.extension().unwrap_or(&OsString::new()) == OsString::from("package") {
							replacement_files.push(subentry_path);
						}
					}
				}
			}
		}
	}
	original_files.sort();
	replacement_files.sort();

	// get all GZPS resources in original package(s)
	let mut gzps_list: Vec<Gzps> = original_files
		.iter()
		.map(|path| {
			let bytes = fs::read(path)?;
			let dbpf = Dbpf::read(&bytes, "")?;
			Ok(dbpf.resources
				.iter()
				.filter_map(|res|
					if let DecodedResource::Gzps(gzps) = res {
						Some(gzps.clone())
					} else {
						None
					})
				.collect::<Vec<_>>())
			})
			.collect::<Result<Vec<Vec<Gzps>>, Box<dyn Error>>>()?
			.into_iter()
			.flatten()
			.collect();
	gzps_list.sort_by_key(|gzps| gzps.name.to_string());
	let pairings: Vec<Option<usize>> = gzps_list.iter().map(|_| None).collect();

	// get all resources from replacement package(s)
	let resources: Vec<DecodedResource> = replacement_files
		.iter()
		.map(|replacement| {
			let bytes = fs::read(replacement)?;
			let new_name = replacement.file_stem().map_or("UNKNOWN".to_string(), |x| x.to_string_lossy().into_owned());
			let dbpf = Dbpf::read(&bytes, &new_name)?;
			Ok(dbpf.resources)
		})
		.collect::<Result<Vec<Vec<DecodedResource>>, Box<dyn Error>>>()?
		.into_iter()
		.flatten()
		.collect();

	// sort replacement resources into outfits
	let mut outfits = Vec::new();
	for resource in &resources {
		if let DecodedResource::Gzps(gzps) = resource {
			let outfit = Outfit::from_resources(gzps.clone(), &resources, true)?;
			outfits.push(outfit);
		}
	}

	// set default output path
	let output_path = if original_files.len() == 1 {
		original_files[0].with_file_name(if let Some(file_name) = original_files[0].file_name() {
			file_name.to_string_lossy().replace(".package", "_DEFAULT.package")
		} else {
			"DEFAULT.package".to_string()
		})
	} else {
		source_dir.join("DEFAULT.package")
	};

	let mut siv = cursive::default();

	let data = SivData {
		output_path,
		gzps_list,
		outfits,
		pairings
	};
	siv.set_user_data(data.clone());

	siv.add_global_callback('q', |s| s.quit());

	siv.add_layer(
		Dialog::around(LinearLayout::horizontal()
				.child(PaddedView::lrtb(0, 2, 0, 0, SelectView::new().with_all(
					data.gzps_list.iter().enumerate().map(|(i, gzps)|
						(gzps.name.to_string(), i)))
					.on_select(update_props)
					.with_name("gzps_select")
					.full_height()
					.scrollable()))
				.child(Panel::new(LinearLayout::vertical()
					.child(TextView::new("Replacement Outfit:"))
					.child(SelectView::<usize>::new()
						.with(|select| { select.add_item("-", 0); })
						.on_submit(set_outfit)
						.popup()
						.with_name("outfit_select")
						.scrollable())
					.child(TextView::new("\n"))
					.child(LinearLayout::horizontal()
						.child(TextView::new("Flags: "))
						.child(TextView::new("").with_name("flags")))
					.child(LinearLayout::horizontal()
						.child(Checkbox::new().on_change(set_show).with_name("show"))
						.child(TextView::new("show "))
						.child(Checkbox::new().on_change(set_townies).with_name("townies"))
						.child(TextView::new("for townies   "))
						.child(Button::new("reset", reset_flags)))
					.child(TextView::new("\nCategory:"))
					.child(LinearLayout::horizontal()
						.child(Checkbox::new().on_change(|s, val|
							set_category(s, Category::Everyday, val)).with_name("everyday"))
						.child(TextView::new("😎 "))
						.child(Checkbox::new().on_change(|s, val|
							set_category(s, Category::Formal, val)).with_name("formal"))
						.child(TextView::new("🎀 "))
						.child(Checkbox::new().on_change(|s, val|
							set_category(s, Category::Undies, val)).with_name("undies"))
						.child(TextView::new("🍑 "))
						.child(Checkbox::new().on_change(|s, val|
							set_category(s, Category::PJs, val)).with_name("pjs"))
						.child(TextView::new("💤 "))
						.child(Checkbox::new().on_change(|s, val|
							set_category(s, Category::Swimwear, val)).with_name("swimwear"))
						.child(TextView::new("🌊 "))
						.child(Checkbox::new().on_change(|s, val|
							set_category(s, Category::Athletic, val)).with_name("athletic"))
						.child(TextView::new("⚽ "))
						.child(Checkbox::new().on_change(|s, val|
							set_category(s, Category::Outerwear, val)).with_name("outerwear"))
						.child(TextView::new("❄️ "))
						.child(Checkbox::new().on_change(|s, val|
							set_category(s, Category::Maternity, val)).with_name("maternity"))
						.child(TextView::new("🫄"))
					)
					.child(TextView::new("\nAge:"))
					.child(LinearLayout::horizontal()
						.child(Checkbox::new().on_change(|s, val|
							set_age(s, Age::Baby, val)).with_name("baby"))
						.child(TextView::new("b "))
						.child(Checkbox::new().on_change(|s, val|
							set_age(s, Age::Toddler, val)).with_name("toddler"))
						.child(TextView::new("p "))
						.child(Checkbox::new().on_change(|s, val|
							set_age(s, Age::Child, val)).with_name("child"))
						.child(TextView::new("c "))
						.child(Checkbox::new().on_change(|s, val|
							set_age(s, Age::Teen, val)).with_name("teen"))
						.child(TextView::new("t "))
						.child(Checkbox::new().on_change(|s, val|
							set_age(s, Age::Adult, val)).with_name("adult"))
						.child(TextView::new("y/a "))
						.child(Checkbox::new().on_change(|s, val|
							set_age(s, Age::Elder, val)).with_name("elder"))
						.child(TextView::new("e")))
					.child(TextView::new("\nGender:"))
					.child(LinearLayout::horizontal()
						.child(Checkbox::new().on_change(|s, val|
							set_gender(s, Gender::Male, val)).with_name("male"))
						.child(TextView::new("m "))
						.child(Checkbox::new().on_change(|s, val|
							set_gender(s, Gender::Female, val)).with_name("female"))
						.child(TextView::new("f")))
					)
					.title("Properties")
					.full_height()
					.full_width()
					.scrollable()))
			.button("Quit", |s| { s.quit(); })
			.button("Save", ask_for_filename)
			.full_screen()
	);

	update_props(&mut siv, &0);

	siv.try_run()?;

	Ok(())
}

fn update_props(s: &mut Cursive, i: &usize) {
	let mut outfit_select = s.find_name::<SelectView::<usize>>("outfit_select").unwrap();

	let mut show_checkbox = s.find_name::<Checkbox>("show").unwrap();
	let mut townies_checkbox = s.find_name::<Checkbox>("townies").unwrap();
	let mut flags_text = s.find_name::<TextView>("flags").unwrap();

	let mut everyday_checkbox = s.find_name::<Checkbox>("everyday").unwrap();
	let mut formal_checkbox = s.find_name::<Checkbox>("formal").unwrap();
	let mut undies_checkbox = s.find_name::<Checkbox>("undies").unwrap();
	let mut pjs_checkbox = s.find_name::<Checkbox>("pjs").unwrap();
	let mut swimwear_checkbox = s.find_name::<Checkbox>("swimwear").unwrap();
	let mut athletic_checkbox = s.find_name::<Checkbox>("athletic").unwrap();
	let mut outerwear_checkbox = s.find_name::<Checkbox>("outerwear").unwrap();
	let mut maternity_checkbox = s.find_name::<Checkbox>("maternity").unwrap();

	let mut baby_checkbox = s.find_name::<Checkbox>("baby").unwrap();
	let mut toddler_checkbox = s.find_name::<Checkbox>("toddler").unwrap();
	let mut child_checkbox = s.find_name::<Checkbox>("child").unwrap();
	let mut teen_checkbox = s.find_name::<Checkbox>("teen").unwrap();
	let mut adult_checkbox = s.find_name::<Checkbox>("adult").unwrap();
	let mut elder_checkbox = s.find_name::<Checkbox>("elder").unwrap();

	let mut male_checkbox = s.find_name::<Checkbox>("male").unwrap();
	let mut female_checkbox = s.find_name::<Checkbox>("female").unwrap();

	s.with_user_data(|data: &mut SivData| {
		let gzps = &data.gzps_list[*i];

		outfit_select.clear();
		outfit_select.add_item("-", usize::MAX);
		let mut select_index = 1;
		for (j, outfit) in data.outfits.iter().enumerate() {
			if Age::are_compatible(&gzps.age, &outfit.gzps.age) && Gender::are_compatible(&gzps.gender, &outfit.gzps.gender, &gzps.age) {
				outfit_select.add_item(&outfit.title, j+1);
				if let Some(outfit_index) = data.pairings[*i] {
					if outfit_index == j {
						outfit_select.set_selection(select_index);
					}
				}
				select_index += 1;
			}
		}

		show_checkbox.set_checked(gzps.flags & 1 == 0);
		townies_checkbox.set_checked(gzps.flags & 8 == 0);
		flags_text.set_content(format!("{}", gzps.flags));

		everyday_checkbox.set_checked(gzps.category.contains(&Category::Everyday));
		formal_checkbox.set_checked(gzps.category.contains(&Category::Formal));
		undies_checkbox.set_checked(gzps.category.contains(&Category::Undies));
		pjs_checkbox.set_checked(gzps.category.contains(&Category::PJs));
		swimwear_checkbox.set_checked(gzps.category.contains(&Category::Swimwear));
		athletic_checkbox.set_checked(gzps.category.contains(&Category::Athletic));
		outerwear_checkbox.set_checked(gzps.category.contains(&Category::Outerwear));
		maternity_checkbox.set_checked(gzps.category.contains(&Category::Maternity));

		baby_checkbox.set_checked(gzps.age.contains(&Age::Baby));
		toddler_checkbox.set_checked(gzps.age.contains(&Age::Toddler));
		child_checkbox.set_checked(gzps.age.contains(&Age::Child));
		teen_checkbox.set_checked(gzps.age.contains(&Age::Teen));
		adult_checkbox.set_checked(gzps.age.contains(&Age::YoungAdult) || gzps.age.contains(&Age::Adult));
		elder_checkbox.set_checked(gzps.age.contains(&Age::Elder));

		male_checkbox.set_checked(gzps.gender.contains(&Gender::Male));
		female_checkbox.set_checked(gzps.gender.contains(&Gender::Female));
	});
}

fn set_outfit(s: &mut Cursive, outfit_index: &usize) {
	let gzps_select = s.find_name::<SelectView::<usize>>("gzps_select").unwrap();
	let i = gzps_select.selected_id().unwrap();
	s.with_user_data(|data: &mut SivData| {
		data.pairings[i] = Some(*outfit_index-1);
	});
}

fn set_show(s: &mut Cursive, value: bool) {
	let gzps_select = s.find_name::<SelectView::<usize>>("gzps_select").unwrap();
	let i = gzps_select.selected_id().unwrap();
	let mut flags_text = s.find_name::<TextView>("flags").unwrap();
	s.with_user_data(|data: &mut SivData| {
		let gzps = &mut data.gzps_list[i];
		if value && gzps.flags & 1 > 0 {
			gzps.flags -= 1;
		} else if !value && gzps.flags & 1 == 0 {
			gzps.flags += 1;
		}
		flags_text.set_content(format!("{}", gzps.flags));
	});
}

fn set_townies(s: &mut Cursive, value: bool) {
	let gzps_select = s.find_name::<SelectView::<usize>>("gzps_select").unwrap();
	let i = gzps_select.selected_id().unwrap();
	let mut flags_text = s.find_name::<TextView>("flags").unwrap();
	s.with_user_data(|data: &mut SivData| {
		let gzps = &mut data.gzps_list[i];
		if value && gzps.flags & 8 > 0 {
			gzps.flags -= 8;
		} else if !value && gzps.flags & 8 == 0 {
			gzps.flags += 8;
		}
		flags_text.set_content(format!("{}", gzps.flags));
	});
}

fn reset_flags(s: &mut Cursive) {
	let gzps_select = s.find_name::<SelectView::<usize>>("gzps_select").unwrap();
	let i = gzps_select.selected_id().unwrap();
	let mut flags_text = s.find_name::<TextView>("flags").unwrap();
	let mut show_checkbox = s.find_name::<Checkbox>("show").unwrap();
	let mut townies_checkbox = s.find_name::<Checkbox>("townies").unwrap();
	s.with_user_data(|data: &mut SivData| {
		let gzps = &mut data.gzps_list[i];
		gzps.flags = 0;
		show_checkbox.set_checked(true);
		townies_checkbox.set_checked(true);
		flags_text.set_content(format!("{}", gzps.flags));
	});
}

fn set_category(s: &mut Cursive, category: Category, value: bool) {
	let gzps_select = s.find_name::<SelectView::<usize>>("gzps_select").unwrap();
	let i = gzps_select.selected_id().unwrap();
	s.with_user_data(|data: &mut SivData| {
		let gzps = &mut data.gzps_list[i];
		Category::toggle_category(&mut gzps.category, category, value);
	});
}

fn set_age(s: &mut Cursive, age: Age, value: bool) {
	let gzps_select = s.find_name::<SelectView::<usize>>("gzps_select").unwrap();
	let i = gzps_select.selected_id().unwrap();
	s.with_user_data(|data: &mut SivData| {
		let gzps = &mut data.gzps_list[i];
		Age::toggle_age(&mut gzps.age, age, value);
	});
}

fn set_gender(s: &mut Cursive, gender: Gender, value: bool) {
	let gzps_select = s.find_name::<SelectView::<usize>>("gzps_select").unwrap();
	let i = gzps_select.selected_id().unwrap();
	s.with_user_data(|data: &mut SivData| {
		let gzps = &mut data.gzps_list[i];
		Gender::toggle_gender(&mut gzps.gender, gender, value);
	});
}

fn ask_for_filename(s: &mut Cursive) {
	let mut output_path = PathBuf::new();
	s.with_user_data(|data: &mut SivData| {
		output_path = data.output_path.clone();
	});
	s.add_layer(
		Dialog::around(
				LinearLayout::vertical()
					.child(EditView::new()
						.content(output_path.to_string_lossy())
						.with_name("filename"))
					.child(LinearLayout::horizontal()
						.child(Checkbox::new().checked().with_name("product_fix"))
						.child(TextView::new("Hide pack icon")))
					.child(LinearLayout::horizontal()
						.child(Checkbox::new().checked().with_name("compress"))
						.child(TextView::new("Compress resources"))))
			.title("Save Default Replacement")
			.button("Cancel", |s| { s.pop_layer(); })
			.button("Ok", save_package)
	);
}

fn save_package(s: &mut Cursive) {
	let filename_str = s.find_name::<EditView>("filename").unwrap().get_content();
	let output_path = PathBuf::from(filename_str.as_str());

	let product_fix = s.find_name::<Checkbox>("product_fix").unwrap().is_checked();
	let compress = s.find_name::<Checkbox>("compress").unwrap().is_checked();

	let mut resources = Vec::new();

	s.with_user_data(|data: &mut SivData| {
		let mut new_outfits = Vec::new();

		let mut text_lists: Vec<TextList> = Vec::new();

		for (i, outfit_index) in data.pairings.iter().enumerate() {
			if let Some(j) = *outfit_index {
				let mut new_gzps = data.gzps_list[i].clone();
				let mut new_outfit = data.outfits[j].clone();

				// copy over shoe/overrides from replacement to original GZPS
				new_gzps.shoe = new_outfit.gzps.shoe;
				new_gzps.overrides = new_outfit.gzps.overrides.clone();

				// enable for young adult + adult
				if new_gzps.age.contains(&Age::YoungAdult) && !new_gzps.age.contains(&Age::Adult) {
					new_gzps.age.push(Age::Adult);
				} else if new_gzps.age.contains(&Age::Adult) && !new_gzps.age.contains(&Age::YoungAdult) {
					new_gzps.age.push(Age::YoungAdult);
				}

				// set product to Base Game to remove pack icon
				if product_fix {
					new_gzps.product = Some(1);
				}

				// update 3IDR's TGIR to match GZPS's TGIR
				new_outfit.idr.id.group_id = new_gzps.id.group_id;
				new_outfit.idr.id.instance_id = new_gzps.id.instance_id;
				new_outfit.idr.id.resource_id = new_gzps.id.resource_id;

				// make young adult clones visible in catalog
				let outfit_name = new_gzps.name.to_string().to_lowercase();
				if outfit_name.starts_with('y') && outfit_name.contains("clone") {
					// create a STR# if none exists with this outfit's group id
					let text_list_id = Identifier::new(TypeId::TextList as u32, new_gzps.id.group_id, 0x1, 0);
					if !text_lists.iter().any(|t| t.id.group_id == new_gzps.id.group_id) {
						text_lists.push(TextList::create_empty(text_list_id.clone()));
					}

					// create BINX resource
					new_outfit.binx = Some(new_outfit.generate_binx());

					// add additional references to 3IDR
					new_outfit.idr.ui_ref = Some(Identifier::new(TypeId::Ui as u32, 0, 0, 0));
					new_outfit.idr.str_ref = Some(text_list_id.clone());
					new_outfit.idr.coll_ref = Some(Identifier::new(TypeId::Coll as u32, 0x0FFEFEFE, 0x0FFE0080, 0));
					new_outfit.idr.gzps_ref = Some(new_gzps.id.clone());

				} else {
					// if not adding BINX, remove unnecessary 3IDR properties
					new_outfit.idr.ui_ref = None;
					new_outfit.idr.str_ref = None;
					new_outfit.idr.coll_ref = None;
					new_outfit.idr.gzps_ref = None;
				}

				// copy new GZPS back to outfit
				new_outfit.gzps = new_gzps;

				// add outfit to list
				new_outfits.push(new_outfit);
			}
		}

		// pull all resources together
		resources = new_outfits
			.iter()
			.flat_map(|o| o.get_resources())
			.collect::<Vec<DecodedResource>>();

		// add any STR# resources that were made
		let text_list_resources = text_lists
			.iter()
			.map(|t| DecodedResource::TextList(t.clone()))
			.collect::<Vec<DecodedResource>>();
		resources.extend_from_slice(&text_list_resources);
	});

	// save package file
	match Dbpf::write_package_file(&resources, &output_path, compress) {
		Ok(_) => {
			s.add_layer(Dialog::around(TextView::new("Success!"))
				.button("Ok", |s| s.quit()));
		}
		Err(why) => {
			s.add_layer(Dialog::around(TextView::new(format!("{}", why)))
				.button("Ok", |s| s.quit()));
		}
	}
}
