use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use rand::Rng;

use crate::dbpf::{ Dbpf, PascalString, SevenBitString };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::mmat::Mmat;
use crate::dbpf::resource_types::txmt::Txmt;
use crate::dbpf::resource_types::txtr::Txtr;
use crate::crc::{ hash_crc24, hash_crc32 };

#[derive(Clone)]
pub struct ObjectRecolor {
	pub mmat: Mmat,
	pub txmt: Txmt,
	pub txtr: Option<Txtr>
}

impl ObjectRecolor {
	pub fn rename(&mut self, object_name: &str, recolor_name: &str) {
		let name = format!("{object_name}-{recolor_name}_{}", self.mmat.subset_name);

		self.mmat.name = PascalString::new(&format!("##0x{:08x}!{}", self.txmt.id.group_id, name));

		let name_txmt = format!("{name}_txmt");
		self.txmt.block.material_definition = SevenBitString::new(&name_txmt);
		self.txmt.block.material_description = SevenBitString::new(&name);
		self.txmt.id.resource_id = hash_crc32(&name_txmt);
		self.txmt.id.instance_id = hash_crc24(&name_txmt);

		if let Some(txtr) = &mut self.txtr {
			let txtr_name = format!("{object_name}-{recolor_name}-{}", self.mmat.subset_name);
			let txtr_name_txtr = format!("{txtr_name}_txtr");
			txtr.name = SevenBitString::new(&txtr_name_txtr);
			txtr.block.file_name = SevenBitString::new(&txtr_name_txtr);
			txtr.id.resource_id = hash_crc32(&txtr_name_txtr);
			txtr.id.instance_id = hash_crc24(&txtr_name_txtr);
			if let Some(txtr_ref) = self.txmt.block.properties.iter_mut().find(|p| p.name.to_string() == "stdMatBaseTextureName") {
				txtr_ref.value = SevenBitString::new(&format!("##0x{:08x}!{}", txtr.id.group_id, txtr_name));
			}
		}
	}
}

pub fn recolor_object(file: PathBuf, title: Option<String>, number: Option<usize>, subset: Option<String>) -> Result<(), Box<dyn Error>> {
	let package = Dbpf::read_from_file(&file, "")?;

	let default_mmats = package.resources.iter().filter_map(|res| {
		if let DecodedResource::Mmat(mmat) = res {
			if mmat.default_material && subset.as_ref().is_none_or(|s| *s == mmat.subset_name.to_string()) {
				println!("Found subset {}", mmat.subset_name);
				return Some(mmat.clone());
			}
		}
		None
	}).collect::<Vec<Mmat>>();

	let default_colors = get_recolors(&package, &default_mmats);

	let number = number.unwrap_or(1);
	let title = title.unwrap_or(file.file_stem().unwrap().to_string_lossy().to_string());

	let resources = make_recolors(&default_colors, number, &title);
	save_recolors(&file, resources, &title)
}

pub fn clone_recolor(file: PathBuf, title: Option<String>, number: Option<usize>, subset: Option<String>) -> Result<(), Box<dyn Error>> {
	let package = Dbpf::read_from_file(&file, "")?;

	let mut subsets = Vec::new();
	let mut mmats = Vec::new();
	for resource in &package.resources {
		if let DecodedResource::Mmat(mmat) = resource {
			if !subsets.contains(&mmat.subset_name) && subset.as_ref().is_none_or(|s| *s == mmat.subset_name.to_string()) {
				println!("Found subset {}", mmat.subset_name);
				subsets.push(mmat.subset_name.clone());
				mmats.push(mmat.clone());
			}
		}
	}

	let base_colors = get_recolors(&package, &mmats);

	let number = number.unwrap_or(1);
	let title = title.unwrap_or(file.file_stem().unwrap().to_string_lossy().to_string());

	let resources = make_recolors(&base_colors, number, &title);
	save_recolors(&file, resources, &title)
}

fn get_recolors(package: &Dbpf, mmats: &[Mmat]) -> Vec<ObjectRecolor> {
	mmats.iter().filter_map(|mmat| {
		let txmt_name = format!("{}_txmt", mmat.name).to_lowercase();
		if let Some(txmt) = package.resources.iter().find_map(|res| {
				if let DecodedResource::Txmt(txmt) = res {
					if txmt.block.material_definition.to_string().to_lowercase() == txmt_name ||
						format!("##0x{:08x}!{}", txmt.id.group_id, txmt.block.material_definition).to_lowercase() == txmt_name {
							return Some(txmt);
					}
				}
				None
			}) {
				let txtr = if let Some(txtr_ref) = txmt.block.properties.iter().find(|p| p.name.to_string() == "stdMatBaseTextureName") {
					let txtr_name = format!("{}_txtr", txtr_ref.value).to_lowercase();
					package.resources.iter().find_map(|res| {
						if let DecodedResource::Txtr(txtr) = res {
							if txtr.name.to_string().to_lowercase() == txtr_name ||
								format!("##0x{:08x}!{}", txtr.id.group_id, txtr.name).to_lowercase() == txtr_name {
									return Some(txtr.clone());
							}
						}
						None
					})
				} else {
					None
				};

				Some(ObjectRecolor {
					mmat: mmat.clone(),
					txmt: txmt.clone(),
					txtr
				})
			} else {
				None
			}
	}).collect::<Vec<ObjectRecolor>>()
}

fn make_recolors(base_colors: &[ObjectRecolor], number: usize, title: &str) -> Vec<DecodedResource> {
	let mut rng = rand::rng();
	let mut resources = Vec::new();
	let mut mmat_id: u32 = 0x00005000;
	for _ in 0..number {
		let guid: u32 = rng.random();
		let mut txtrs_used: HashMap<SevenBitString, SevenBitString> = HashMap::new();
		for color in base_colors {
			let mut new_color = color.clone();

			new_color.mmat.id.group_id = 0xFFFFFFFF;
			new_color.mmat.id.resource_id = 0x00000000;
			new_color.mmat.id.instance_id = mmat_id;
			mmat_id += 1;
			new_color.mmat.default_material = false;

			new_color.txmt.id.group_id =  0x1C050000;

			if let Some(txtr) = &mut new_color.txtr {
				txtr.id.group_id = 0x1C050000;
			}

			new_color.rename(&title.replace(' ', ".").replace('_', ".").replace('-', "."), &format!("{:08x}", guid));

			if let Some(txtr_ref_og) = color.txmt.block.properties.iter().find(|p| p.name.to_string() == "stdMatBaseTextureName") {
				if let Some(txtr_ref_new) = new_color.txmt.block.properties.iter_mut().find(|p| p.name.to_string() == "stdMatBaseTextureName") {
					if let Some(used_txtr) = txtrs_used.get(&txtr_ref_og.value) {
						txtr_ref_new.value = used_txtr.clone();
						new_color.txtr = None;
					} else {
						txtrs_used.insert(txtr_ref_og.value.clone(), txtr_ref_new.value.clone());
					}
				}
			}

			resources.push(DecodedResource::Mmat(new_color.mmat));
			resources.push(DecodedResource::Txmt(new_color.txmt));
			if let Some(txtr) = new_color.txtr {
				resources.push(DecodedResource::Txtr(txtr));
			}
		}
	}
	resources
}

fn save_recolors(file: &PathBuf, resources: Vec<DecodedResource>, title: &str) -> Result<(), Box<dyn Error>> {
	let mut package = Dbpf::new(resources)?;
	package.is_compressed = true;
	package.write_to_file(&file.with_file_name(format!("{title}_RECS.package")))
}
