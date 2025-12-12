use std::error::Error;
use std::path::{ Path, PathBuf };
use rand::Rng;

use crate::dbpf::{ Dbpf, Identifier, TypeId };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::text_list::{ TextList, StringItem };
use crate::dbpf::resource_types::binx::Binx;
use crate::dbpf::resource_types::gzps::Gzps;
use crate::dbpf::resource_types::idr::Idr;
use crate::dbpf::resource_types::txmt::Txmt;
use crate::dbpf::resource_types::txtr::Txtr;

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

pub fn recolor_outfit(files: Vec<PathBuf>, title: Option<String>, number: Option<usize>, repo: bool) -> Result<(), Box<dyn Error>> {
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
			let mut recolor = create_outfit_recolor(&template, &title, i);
			if repo {
				if is_main {
					main_refs.push(recolor.idr.txmt_refs.clone());
				} else {
					recolor.idr.txmt_refs = main_refs[i].clone();
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

fn create_outfit_recolor(template: &OutfitRecolor, title: &str, i: usize) -> OutfitRecolor {
	let mut rng = rand::rng();
	let guid: u32 = rng.random();

	let mut gzps = template.gzps.clone();
	gzps.id.group_id = guid;

	let idr = template.idr.replace_guid(guid);

	let mut binx = template.binx.clone();
	binx.id.group_id = guid;
	binx.sort_index += i as i32 + 1;

	let text_list = TextList {
		id: Identifier {
			type_id: TypeId::TextList,
			group_id: guid,
			instance_id: 1,
			resource_id: 0,
		},
		key_name: [0; 64],
		strings: vec![
			StringItem {
				language_code: 0x01,
				title: format!("{title}_{:02}", i+1),
				description: "".to_string()
			}
		]
	};

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
