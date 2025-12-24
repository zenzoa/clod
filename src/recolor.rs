use std::error::Error;
use std::path::{ Path, PathBuf };
use rand::Rng;

use crate::dbpf::{ Dbpf, Identifier, TypeId };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::text_list::TextList;
use crate::dbpf::resource_types::binx::Binx;
use crate::dbpf::resource_types::gzps::{ Gzps, OutfitSpec, Part, Age, Gender, Category, Shoe };
use crate::dbpf::resource_types::idr::Idr;
use crate::dbpf::resource_types::txmt::Txmt;
use crate::dbpf::resource_types::txtr::{ Txtr, TxtrPurpose };
use crate::crc::hash_crc32;

#[derive(Clone)]
pub struct OutfitRecolor {
	pub gzps: Gzps,
	pub idr: Idr,
	pub binx: Binx,
	pub text_list: TextList,
	pub txmts: Vec<Txmt>,
	pub txtrs: Vec<Txtr>
}

impl OutfitRecolor {
	pub fn to_package(&self) -> Result<Dbpf, Box<dyn Error>> {
		let mut resources = vec![
			DecodedResource::Gzps(self.gzps.clone()),
			DecodedResource::Idr(self.idr.clone()),
			DecodedResource::Binx(self.binx.clone()),
			DecodedResource::TextList(self.text_list.clone())
		];
		for txmt in &self.txmts {
			resources.push(DecodedResource::Txmt(txmt.clone()));
		}
		for txtr in &self.txtrs {
			resources.push(DecodedResource::Txtr(txtr.clone()));
		}
		let mut package = Dbpf::new(resources)?;
		package.is_compressed = true;
		Ok(package)
	}

	pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
		self.to_package()?.write_to_file(path)
	}
}

pub fn recolor_outfit_from_template(files: Vec<PathBuf>, title: Option<String>, number: Option<usize>, repo: bool) -> Result<(), Box<dyn Error>> {
	let mut main_refs = Vec::new();
	let title = title.unwrap_or("OutfitRecolor".to_string());
	for file in files {
		let package = Dbpf::read_from_file(&file, "")?;
		let template = create_outfit_template(&package)?;
		let is_main = main_refs.is_empty();
		for i in 0..number.unwrap_or(1) {
			let gender_str = template.gzps.age_gender_string();
			let repo_str = if is_main || !repo { "" } else { "_REPO" };
			let recolor_title = format!("{title}{gender_str}_{:02}{repo_str}", i+1);
			let mut recolor = create_outfit_recolor_from_template(&template, &title, i);
			if repo {
				if is_main {
					main_refs.push(recolor.clone());
				} else {
					recolor.idr.txmt_refs = main_refs[i].idr.txmt_refs.clone();
					recolor.binx.sort_index = main_refs[i].binx.sort_index.clone();
					recolor.txmts = Vec::new();
					recolor.txtrs = Vec::new();
				}
			}
			let path = file.with_file_name(format!("{recolor_title}.package"));
			recolor.save(&path)?;
		}
	}

	Ok(())
}

fn create_outfit_recolor_from_template(template: &OutfitRecolor, title: &str, i: usize) -> OutfitRecolor {
	let mut rng = rand::rng();
	let guid: u32 = rng.random();

	let mut gzps = template.gzps.clone();
	gzps.id.group_id = guid;

	let idr = template.idr.replace_guid(guid);

	let mut binx = template.binx.clone();
	binx.id.group_id = guid;
	binx.sort_index += i as i32 + 1;

	let text_list = TextList::from_string(&format!("{title}_{:02}", i+1), guid);

	let txtrs = template.txtrs.iter().map(|txtr| txtr.replace_guid(guid)).collect::<Vec<Txtr>>();

	let txmts = template.txmts.iter().map(|txmt| txmt.replace_guid(guid)).collect::<Vec<Txmt>>();

	OutfitRecolor {
		gzps,
		idr,
		binx,
		text_list,
		txmts,
		txtrs
	}
}

fn create_outfit_template(package: &Dbpf) -> Result<OutfitRecolor, Box<dyn Error>> {
	let gzps = package.resources.iter()
		.find_map(|r| if let DecodedResource::Gzps(gzps) = r { Some(gzps.clone()) } else { None })
		.ok_or("Unable to find GZPS")?;

	let idr = package.resources.iter()
		.find_map(|r| if let DecodedResource::Idr(idr) = r { Some(idr.clone()) } else { None })
		.ok_or("Unable to find 3IDR")?;

	let binx = package.resources.iter()
		.find_map(|r| if let DecodedResource::Binx(binx) = r { Some(binx.clone()) } else { None })
		.ok_or("Unable to find BINX")?;

	let text_list = package.resources.iter()
		.find_map(|r| if let DecodedResource::TextList(text_list) = r { Some(text_list.clone()) } else { None })
		.ok_or("Unable to find STR")?;

	let txmts: Vec<Txmt> = package.resources.iter()
		.filter_map(|r| if let DecodedResource::Txmt(txmt) = r { Some(txmt.clone()) } else { None })
		.collect();

	let txtrs: Vec<Txtr> = package.resources.iter()
		.filter_map(|r| if let DecodedResource::Txtr(txtr) = r { Some(txtr.clone()) } else { None })
		.collect();

	Ok(OutfitRecolor {
		gzps,
		idr,
		binx,
		text_list,
		txmts,
		txtrs
	})
}

pub fn recolor_outfit_from_mesh(file: PathBuf, title: Option<String>, number: Option<usize>, part: String, age_gender: String, category: Option<String>, shoe: Option<String>) -> Result<(), Box<dyn Error>> {
	let package = Dbpf::read_from_file(&file, "")?;

	let cres_id = package.resources.iter().find_map(|r| {
		if let DecodedResource::Cres(cres) = r {
			Some(cres.id.clone())
		} else {
			None
		}
	}).ok_or("Package file does not contain CRES resource")?;

	let shpe_id = package.resources.iter().find_map(|r| {
		if let DecodedResource::Shpe(shpe) = r {
			Some(shpe.id.clone())
		} else {
			None
		}
	}).ok_or("Package file does not contain SHPE resource")?;

	let subsets = package.resources.iter().find_map(|r| {
		if let DecodedResource::Shpe(shpe) = r {
			Some(shpe.block.materials.iter().map(|m| m.subset.to_string()).collect::<Vec<String>>())
		} else {
			None
		}
	}).ok_or("Package file does not contain SHPE resource")?;

	let filename = file.file_stem().unwrap().to_string_lossy().replace("_MESH", "").replace("_Mesh", "").replace("_mesh", "");
	let title = title.unwrap_or(filename.clone());

	let parts = Part::from_string(&part);

	let (age_string, gender_string) = age_gender.split_at(1);
	let ages = Age::from_string(age_string);
	let genders = Gender::from_string(gender_string);

	let categories = category.map_or(vec![], |c| Category::from_string(&c));

	let shoe = shoe.map_or_else(|| match parts[0] {
		Part::Bottom | Part::Body => Shoe::Normal,
		_ => Shoe::None
	}, |s| Shoe::from_string(&s));

	let mut rng = rand::rng();

	for i in 0..number.unwrap_or(1) {
		let spec = OutfitSpec {
			guid: rng.random(),
			name: title.replace(' ', "_").to_lowercase(),
			parts: parts.clone(),
			ages: ages.clone(),
			genders: genders.clone(),
			categories: categories.clone(),
			flags: 0x8,
			shoe,
			subsets: subsets.clone()
		};
		let recolor = create_outfit_recolor_from_mesh(&spec, &cres_id, &shpe_id, i);
		let path = file.with_file_name(format!("{filename}_{:02}.package", i+1));
		recolor.save(&path)?;
	}

	Ok(())
}

fn create_outfit_recolor_from_mesh(spec: &OutfitSpec, cres_id: &Identifier, shpe_id: &Identifier, i: usize) -> OutfitRecolor {
	let mut gzps = spec.to_gzps();
	gzps.id.group_id = spec.guid;
	gzps.id.resource_id = 0;
	gzps.id.instance_id = 1;

	let mut binx = Binx::from_gzps(&gzps);
	binx.sort_index = (hash_crc32(&spec.name.to_string()) + i as u32) as i32;

	let numbered_name = format!("{}_{:02}", spec.name, i+1);
	let text_list = TextList::from_string(&numbered_name, spec.guid);
	let resource_title = numbered_name.replace('_', ".");

	let txmts = spec.subsets.iter().map(|subset| {
		match subset.as_str() {
			"top" | "bottom" | "body" =>
				Txmt::create_textureless(spec.guid, &format!("{resource_title}-{subset}"), "SimSkin"),
			_ =>
				Txmt::create_textured(&format!(
					"##0x{:08x}!{resource_title}-{subset}_txtr", spec.guid),
					spec.guid,
					&format!("{resource_title}-{subset}"),
					"SimStandardMaterial"
				)
		}
	}).collect::<Vec<Txmt>>();

	let txtrs = spec.subsets.iter().filter_map(|subset| {
		match subset.as_str() {
			"top" | "bottom" | "body" => None,
			_ => Some(Txtr::create_empty(spec.guid, &format!("{resource_title}-{subset}"), TxtrPurpose::Outfit))
		}
	}).collect::<Vec<Txtr>>();

	let idr = Idr {
		id: Identifier::new(u32::from(TypeId::Idr), spec.guid, 0, 1),
		cres_ref: Some(cres_id.clone()),
		shpe_ref: Some(shpe_id.clone()),
		txmt_refs: txmts.iter().map(|txmt| txmt.id.clone()).collect(),
		ui_ref: Some(Identifier::new(u32::from(TypeId::Idr), spec.guid, 0, 1)),
		str_ref: Some(text_list.id.clone()),
		coll_ref: Some(Identifier::new(u32::from(TypeId::Coll), 0x0FFEFEFE, 0x00000000, 0x0FFE0080)),
		gzps_ref: Some(gzps.id.clone())
	};

	OutfitRecolor {
		gzps,
		idr,
		binx,
		text_list,
		txmts,
		txtrs
	}
}
