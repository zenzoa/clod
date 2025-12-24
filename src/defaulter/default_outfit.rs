use std::error::Error;
use std::path::{ Path, PathBuf };
use std::fs;

use cursive::{ Cursive, With };
use cursive::view::{ Nameable, Scrollable, Resizable };
use cursive::views::{ Dialog, DialogFocus, Button, TextView, EditView, Checkbox, SelectView, LinearLayout, Panel, PaddedView };

use crate::dbpf::{ Dbpf, Identifier, TypeId, PascalString };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Age, Gender, Category };
use crate::dbpf::resource_types::text_list::TextList;
use crate::outfit::Outfit;

use super::{ get_default_replacement_files, extract_resources, extract_gzps, default_output_path };

#[derive(Clone, Default)]
struct GzpsSettings {
	hide_pack_icon: bool,
	unisex: bool,
	hidden: Option<bool>,
	notownies: Option<bool>,
	categories: Option<Vec<Category>>
}

impl GzpsSettings {
	fn new(hide_pack_icon: bool) -> Self {
		Self {
			hide_pack_icon,
			..Default::default()
		}
	}

	fn from_string(hide_pack_icon: bool, s: &str) -> Self {
		Self {
			hide_pack_icon,
			unisex: s.contains("unisex"),
			hidden: Some(s.contains("hidden")),
			notownies: Some(s.contains("notownies")),
			categories: Some(Category::from_string(s))
		}
	}

	fn apply(&self, gzps: &mut Gzps) {
		// set product to Base Game to remove pack icon
		if self.hide_pack_icon {
			gzps.product = Some(1);
		}

		// enable for all genders (if baby, toddler, or child)
		if self.unisex {
			gzps.make_unisex();
		}

		// enable for young adult + adult
		if gzps.ages.contains(&Age::YoungAdult) && !gzps.ages.contains(&Age::Adult) {
			gzps.ages.push(Age::Adult);
		} else if gzps.ages.contains(&Age::Adult) && !gzps.ages.contains(&Age::YoungAdult) {
			gzps.ages.push(Age::YoungAdult);
		}

		// set hidden/visible in CAS
		if let Some(hidden) = self.hidden {
			if hidden && gzps.flags & 1 == 0 {
				gzps.flags += 1;
			} else if !hidden && gzps.flags & 1 > 0 {
				gzps.flags -= 1;
			}
		}

		// set enabled/disabled for townies
		if let Some(notownies) = self.notownies {
			if notownies && gzps.flags & 8 == 0 {
				gzps.flags += 8;
			} else if !notownies && gzps.flags & 8 > 0 {
				gzps.flags -= 8;
			}
		}

		// set categories
		if let Some(categories) = &self.categories {
			gzps.categories = categories.clone();
		}
	}
}

#[derive(Clone, Default)]
struct SivData {
	source_dir: PathBuf,
	gzps_settings: GzpsSettings,
	gzps_list: Vec<Gzps>,
	outfits: Vec<Outfit>,
	pairings: Vec<Option<usize>>
}

pub fn default_outfit(source: Option<PathBuf>, auto: bool, hide_pack_icon: bool) -> Result<(), Box<dyn Error>> {
	let source_dir = source.unwrap_or(PathBuf::from("./"));

	print!("Reading files...");
	let (original_files, replacement_files) = get_default_replacement_files(&source_dir)?;
	println!("DONE");

	print!("Extracting resources...");

	// get all GZPS resources in original package(s)
	let gzps_list = extract_gzps(&original_files)?;

	// get all resources from replacement package(s)
	let resources = extract_resources(&replacement_files)?;

	// sort replacement resources into outfits
	let mut outfits = Vec::new();
	for resource in &resources {
		if let DecodedResource::Gzps(gzps) = resource {
			let outfit = Outfit::from_resources(gzps.clone(), &resources, true)?;
			outfits.push(outfit);
		}
	}
	outfits.sort_by_key(|o| o.title.clone());

	// set up initial pairings
	let mut outfit_indexes: Vec<usize> = outfits.iter().enumerate().map(|(i, _)| i).collect();
	let pairings: Vec<Option<usize>> = gzps_list.iter().map(|gzps| {
		let mut pairing = None;
		let mut index_to_remove = None;
		for (j, outfit_index) in outfit_indexes.iter().enumerate() {
			let outfit = &outfits[*outfit_index];
			if Age::are_compatible(&gzps.ages, &outfit.gzps.ages) && Gender::are_compatible(&gzps.genders, &outfit.gzps.genders, &gzps.ages) {
				pairing = Some(*outfit_index);
				index_to_remove = Some(j);
				break;
			}
		}
		if let Some(index_to_remove) = index_to_remove {
			outfit_indexes.remove(index_to_remove);
		}
		pairing
	}).collect();

	// look for property override file
	let mut gzps_settings = GzpsSettings::new(hide_pack_icon);
	for entry in (fs::read_dir(&source_dir)?).flatten() {
		let entry_path = entry.path();
		if entry_path.is_file() && entry_path.extension().is_some_and(|ext| ext == "properties") {
			if let Some(prop_string) = entry_path.file_stem() {
				gzps_settings = GzpsSettings::from_string(hide_pack_icon, &prop_string.to_string_lossy());
			}
		}
	}

	println!("DONE");

	let data = SivData {
		source_dir,
		gzps_settings,
		gzps_list,
		outfits,
		pairings
	};

	if auto {
		print!("Saving default replacement...");
		let output_path = default_output_path(&data.source_dir, "DEFAULT");
		let resources = save_default(&data, &output_path, true)?;
		println!("DONE");

		print!("Saving extras...");
		save_extras(&data, &resources)?;
		println!("DONE");

		Ok(())

	} else {
		run_ui(data)
	}
}

fn run_ui(data: SivData) -> Result<(), Box<dyn Error>> {
	let mut siv = cursive::default();
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
						.child(Button::new("set to 0", reset_flags)))
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
			if Age::are_compatible(&gzps.ages, &outfit.gzps.ages) && Gender::are_compatible(&gzps.genders, &outfit.gzps.genders, &gzps.ages) {
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

		everyday_checkbox.set_checked(gzps.categories.contains(&Category::Everyday));
		formal_checkbox.set_checked(gzps.categories.contains(&Category::Formal));
		undies_checkbox.set_checked(gzps.categories.contains(&Category::Undies));
		pjs_checkbox.set_checked(gzps.categories.contains(&Category::PJs));
		swimwear_checkbox.set_checked(gzps.categories.contains(&Category::Swimwear));
		athletic_checkbox.set_checked(gzps.categories.contains(&Category::Athletic));
		outerwear_checkbox.set_checked(gzps.categories.contains(&Category::Outerwear));
		maternity_checkbox.set_checked(gzps.categories.contains(&Category::Maternity));

		baby_checkbox.set_checked(gzps.ages.contains(&Age::Baby));
		toddler_checkbox.set_checked(gzps.ages.contains(&Age::Toddler));
		child_checkbox.set_checked(gzps.ages.contains(&Age::Child));
		teen_checkbox.set_checked(gzps.ages.contains(&Age::Teen));
		adult_checkbox.set_checked(gzps.ages.contains(&Age::YoungAdult) || gzps.ages.contains(&Age::Adult));
		elder_checkbox.set_checked(gzps.ages.contains(&Age::Elder));

		male_checkbox.set_checked(gzps.genders.contains(&Gender::Male));
		female_checkbox.set_checked(gzps.genders.contains(&Gender::Female));
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
		Category::toggle_category(&mut gzps.categories, category, value);
	});
}

fn set_age(s: &mut Cursive, age: Age, value: bool) {
	let gzps_select = s.find_name::<SelectView::<usize>>("gzps_select").unwrap();
	let i = gzps_select.selected_id().unwrap();
	s.with_user_data(|data: &mut SivData| {
		let gzps = &mut data.gzps_list[i];
		Age::toggle_age(&mut gzps.ages, age, value);
	});
}

fn set_gender(s: &mut Cursive, gender: Gender, value: bool) {
	let gzps_select = s.find_name::<SelectView::<usize>>("gzps_select").unwrap();
	let i = gzps_select.selected_id().unwrap();
	s.with_user_data(|data: &mut SivData| {
		let gzps = &mut data.gzps_list[i];
		Gender::toggle_gender(&mut gzps.genders, gender, value);
	});
}

fn ask_for_filename(s: &mut Cursive) {
	let mut output_path = PathBuf::new();
	s.with_user_data(|data: &mut SivData| {
		output_path = default_output_path(&data.source_dir, "DEFAULT");
	});
	s.add_layer(
		Dialog::around(
				LinearLayout::vertical()
					.child(EditView::new()
						.content(output_path.to_string_lossy())
						.on_submit(|s, _| {
							let mut filename_dialog = s.find_name::<Dialog>("filename_dialog").unwrap();
							let _ = filename_dialog.set_focus(DialogFocus::Button(1));
						})
						.with_name("filename")
						.min_width(40))
					.child(LinearLayout::horizontal()
						.child(Checkbox::new().checked().with_name("compress"))
						.child(TextView::new("Compress resources")))
					.child(LinearLayout::horizontal()
						.child(Checkbox::new().checked().with_name("product_fix"))
						.child(TextView::new("Hide pack icon")))
					.child(LinearLayout::horizontal()
						.child(Checkbox::new().checked().with_name("add_extras"))
						.child(TextView::new("Save extra outfits"))))
			.padding_lrtb(2, 2, 1, 0)
			.title("Save Default Replacement")
			.button("Cancel", |s| { s.pop_layer(); })
			.button("Ok", save_package)
			.with_name("filename_dialog")
	);
}

fn save_package(s: &mut Cursive) {
	s.add_layer(Dialog::around(TextView::new("Saving...")));

	let filename = s.find_name::<EditView>("filename").unwrap().get_content();
	let output_path = PathBuf::from(filename.as_str());

	let compress = s.find_name::<Checkbox>("compress").unwrap().is_checked();
	let hide_pack_icon = s.find_name::<Checkbox>("product_fix").unwrap().is_checked();
	let add_extras = s.find_name::<Checkbox>("add_extras").unwrap().is_checked();

	let mut save_result = Ok(());
	let mut save_extras_result = None;

	s.with_user_data(|data: &mut SivData| {
		data.gzps_settings.hide_pack_icon = hide_pack_icon;

		match save_default(data, &output_path, compress) {
			Ok(resources) => {
				save_result = Ok(());
				if add_extras {
					save_extras_result = Some(save_extras(data, &resources));
				}
			}
			Err(why) => {
				save_result = Err(why);
			}
		}
	});

	// share the result
	match (save_result, save_extras_result) {
		(Ok(_), None) | (Ok(_), Some(Ok(_))) => {
			s.add_layer(Dialog::around(TextView::new("Success!"))
				.button("Ok", |s| s.quit()));
		}
		(Err(why), None) | (Err(why), Some(Ok(_))) => {
			s.add_layer(Dialog::around(TextView::new(format!("Unable to save default: {why}")))
				.button("Ok", |s| s.quit()));
		}
		(Ok(_), Some(Err(why))) => {
			s.add_layer(Dialog::around(TextView::new(format!("Unable to save extras: {why}")))
				.button("Ok", |s| s.quit()));
		}
		(Err(why), Some(Err(why2))) => {
			s.add_layer(Dialog::around(TextView::new(format!("Unable to save default: {why}\nUnable to save extras: {why2}")))
				.button("Ok", |s| s.quit()));
		}
	}
}

fn save_default(data: &SivData, output_path: &Path, compress: bool) -> Result<Vec<DecodedResource>, Box<dyn Error>> {
	let mut new_outfits = Vec::new();
	let mut text_lists: Vec<TextList> = Vec::new();

	for (i, outfit_index) in data.pairings.iter().enumerate() {
		if let Some(j) = *outfit_index {
			let mut new_gzps = data.gzps_list[i].clone();
			let mut new_outfit = data.outfits[j].clone();

			// copy over shoe/overrides from replacement to original GZPS
			new_gzps.shoe = new_outfit.gzps.shoe;
			new_gzps.overrides = new_outfit.gzps.overrides.clone();

			// apply settings
			data.gzps_settings.apply(&mut new_gzps);

			// update 3IDR's TGIR to match GZPS's TGIR
			new_outfit.idr.id.group_id = new_gzps.id.group_id;
			new_outfit.idr.id.instance_id = new_gzps.id.instance_id;
			new_outfit.idr.id.resource_id = new_gzps.id.resource_id;

			// make young adult clones visible in catalog
			let outfit_name = new_gzps.name.to_string().to_lowercase();
			if outfit_name.starts_with('y') && outfit_name.contains("clone") {
				// create a STR# if none exists with this outfit's group id
				let text_list_id = Identifier::new(u32::from(TypeId::TextList), new_gzps.id.group_id, 0, 1);
				if !text_lists.iter().any(|t| t.id.group_id == new_gzps.id.group_id) {
					text_lists.push(TextList::create_empty(text_list_id.clone()));
				}

				// create BINX resource
				new_outfit.generate_binx();

				// add additional references to 3IDR
				new_outfit.idr.ui_ref = Some(Identifier::new(u32::from(TypeId::Ui), 0, 0, 0));
				new_outfit.idr.str_ref = Some(text_list_id.clone());
				new_outfit.idr.coll_ref = Some(Identifier::new(u32::from(TypeId::Coll), 0x0FFEFEFE, 0, 0x0FFE0080));
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

	Dbpf::write_package_file(&resources, &output_path, compress)?;

	Ok(resources)
}

fn save_extras(data: &SivData, resources: &[DecodedResource]) -> Result<(), Box<dyn Error>> {
	let mut extra_outfits: Vec<Outfit> = data.outfits.iter().enumerate()
		.filter(|(i, _)| !data.pairings.iter().any(|p| p.is_some_and(|j| j == *i)))
		.map(|(_, outfit)| outfit.clone())
		.collect();

	let mut text_lists: Vec<TextList> = Vec::new();

	for outfit in extra_outfits.iter_mut() {
		// create a STR# if none exists with this outfit's group id
		let text_list_id = Identifier::new(u32::from(TypeId::TextList), outfit.gzps.id.group_id, 0, 1);
		if !text_lists.iter().any(|t| t.id.group_id == outfit.gzps.id.group_id) {
			text_lists.push(TextList::create_empty(text_list_id.clone()));
		}

		// decustomize
		outfit.gzps.creator = PascalString::new("00000000-0000-0000-0000-000000000000");

		// apply settings
		data.gzps_settings.apply(&mut outfit.gzps);

		// create BINX resource
		outfit.generate_binx();
	}

	// collect resources together
	let mut extra_resources: Vec<DecodedResource> = extra_outfits.iter().flat_map(|outfit| {
		outfit.get_resources()
	}).collect();

	// add any STR# resources that were made
	let text_list_resources = text_lists
		.iter()
		.map(|t| DecodedResource::TextList(t.clone()))
		.collect::<Vec<DecodedResource>>();
	extra_resources.extend_from_slice(&text_list_resources);

	// remove dupes
	extra_resources = extra_resources.into_iter()
		.filter(|r| !resources.iter().any(|r2| r2.get_id() == r.get_id()))
		.collect();

	// save package
	if !extra_resources.is_empty() {
		let extra_path = default_output_path(&data.source_dir, "EXTRAS");
		Dbpf::write_package_file(&extra_resources, &extra_path, true)?;
	}

	Ok(())
}
