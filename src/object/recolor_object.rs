use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::dbpf::{ Dbpf, PascalString, SevenBitString, TypeId };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::mmat::Mmat;
use crate::dbpf::resource_types::txmt::Txmt;
use crate::dbpf::resource_types::txtr::{Txtr, TxtrPurpose};
use crate::crc::{ hash_crc24, hash_crc32 };
use crate::helpers::{ dehash_name, get_resource_by_name };

struct ObjectRecolor {
	name: String,
	states: Vec<ObjectRecolorState>
}

struct ObjectRecolorState {
	subset_name: String,
	state_name: String,
	mmat: Option<Mmat>,
	txmt: Option<Txmt>,
	txtr: Option<Txtr>
}

pub fn clone_recolor(file: PathBuf, output: Option<PathBuf>, old_name: String, new_name: String, multiple: Option<usize>) -> Result<(), Box<dyn Error>> {
	let mut rng = rand::rng();
	let object = Dbpf::read_from_file(&file)?;

	let count = multiple.unwrap_or(1);
	for i in 0..count {
		let this_new_name = if count > 1 {
			format!("{new_name}{:02}", i + 1)
		} else {
			new_name.clone()
		};
		let mut new_families: HashMap<String, PascalString> = HashMap::new();
		let mut new_resources = object.resources.iter().cloned().collect::<Vec<DecodedResource>>();
		for resource in new_resources.iter_mut() {
			match resource {
				DecodedResource::Mmat(mmat) => {
					if let Some(family) = new_families.get(&mmat.family.to_string()) {
						mmat.family = family.clone();
					} else {
						let family = new_family(&mut rng);
						new_families.insert(mmat.family.to_string(), family.clone());
						mmat.family = family;
					}
					mmat.rename(&old_name, &this_new_name);
					println!("Creating {}", mmat.name);
				}
				DecodedResource::Txmt(txmt) => {
					txmt.rename(&old_name, &this_new_name);
					println!("Creating {}", txmt.block.material_definition);
				}
				DecodedResource::Txtr(txtr) => {
					txtr.rename(&old_name, &this_new_name);
					println!("Creating {}", txtr.name);
				}
				_ => {}
			}
		}

		let output_path = match &output {
			Some(output) =>
				if multiple.is_none_or(|m| m == 1) {
					output.with_extension("package")
				} else {
					output.with_file_name(&format!("{}{:02}.package", output.file_stem().unwrap().to_string_lossy(), i + 1))
				},
			None => {
				let title = file.file_stem().unwrap().to_string_lossy();
				file.with_file_name(&format!("{title}_REC{:02}.package", i + 1))
			}
		};
		println!("Saving {}", output_path.to_string_lossy());
		Dbpf::write_package_file(&new_resources, &output_path)?;

		if i < count - 1 {
			println!("");
		}
	}
	Ok(())
}

pub fn recolor_object(file: PathBuf, output: Option<PathBuf>, name: Option<String>, subset: Option<String>, multiple: Option<usize>) -> Result<(), Box<dyn Error>> {
	let object = Dbpf::read_from_file(&file)?;

	// find repo'd subsets
	let repo_subsets = get_repo_subsets(&object.resources);

	// get default mmats
	let mut default_mmats = get_mmats(&object.resources, true, &subset);
	if default_mmats.is_empty() {
		default_mmats = get_mmats(&object.resources, false, &subset);
	}

	let default_recolors = default_mmats.iter()
		.filter_map(|mmat| {
			let mmats_in_family = get_mmats_in_family(&object.resources, &mmat.family, &mmat.subset_name);
			let mut states = Vec::new();
			for mmat in mmats_in_family {
				let txmt_name = format!("{}_txmt", dehash_name(&mmat.name.to_string()));
				println!("Found MMAT \"{}\"", txmt_name);
				if let Some(DecodedResource::Txmt(txmt)) = get_resource_by_name(&txmt_name, TypeId::Txmt, &object.resources) {
					println!("  Found TXMT \"{}\"", txmt_name);
					let state_name = match mmat.object_state_index {
						-1 => "".to_string(),
						_ => match mmat.name.to_string().split('_').last() {
							Some(s) => format!("_{s}"),
							None => "".to_string()
						}
					};
					states.push(ObjectRecolorState {
						subset_name: mmat.subset_name.to_string(),
						state_name: state_name.clone(),
						mmat: Some(mmat.clone()),
						txmt: Some(txmt.clone()),
						txtr: get_linked_txtr(&txmt, &object.resources)
					});
					for (source_subset, repo_subset_name) in &repo_subsets {
						if *source_subset == mmat.subset_name.to_string() {
							let repo_txmt_name = txmt_name.replace(source_subset, repo_subset_name);
							if let Some(DecodedResource::Txmt(repo_txmt)) = get_resource_by_name(&repo_txmt_name, TypeId::Txmt, &object.resources) {
								println!("  Found TXMT \"{}\"", repo_txmt_name);
								states.push(ObjectRecolorState {
									subset_name: repo_subset_name.clone(),
									state_name: state_name.clone(),
									mmat: None,
									txmt: Some(repo_txmt.clone()),
									txtr: get_linked_txtr(&repo_txmt, &object.resources)
								});
							}

						}
					}
				}
			};
			if !states.is_empty() {
				let object_name = dehash_name(&mmat.name.to_string())
					.split_once('_')
					.map(|(s, _)| s.to_string())
					.unwrap_or(mmat.name.to_string());
				Some(ObjectRecolor {
					name: object_name,
					states
				})
			} else {
				None
			}
		})
		.collect::<Vec<ObjectRecolor>>();

	for i in 0..multiple.unwrap_or(1) {
		let recolor_name = if multiple.is_none_or(|m| m == 1) {
			name.clone()
		} else {
			name.as_ref().map(|n| format!("{n}{:02}", i + 1))
		};
		let resources = make_recolor(&recolor_name, &default_recolors, 0x00005000);
		let output_path = match &output {
			Some(output) =>
				if multiple.is_none_or(|m| m == 1) {
					output.with_extension("package")
				} else {
					output.with_file_name(&format!("{}{:02}.package", output.file_stem().unwrap().to_string_lossy(), i + 1))
				},
			None => {
				let title = file.file_stem().unwrap().to_string_lossy();
				file.with_file_name(&format!("{title}_REC{:02}.package", i + 1))
			}
		};
		println!("Saving {}", output_path.to_string_lossy());
		Dbpf::write_package_file(&resources, &output_path)?;
	}

	Ok(())
}

fn make_recolor(name: &Option<String>, default_recolors: &[ObjectRecolor], first_mmat_instance: u32) -> Vec<DecodedResource> {
	let mut rng = rand::rng();
	let id: u32 = rng.random();

	let recolor_name = name.clone().unwrap_or(format!("{:08x}", id));

	let group_id = 0x1C050000;
	let mut mmat_instance = first_mmat_instance;

	let mut resources = Vec::new();

	for default_recolor in default_recolors {
		let family = new_family(&mut rng);
		for recolor_state in &default_recolor.states {
			let base_name = format!("{}_{}_{}{}", default_recolor.name, recolor_state.subset_name, recolor_name, recolor_state.state_name);
			let base_name_with_group = format!("##0x{:08x}!{}", group_id, base_name);

			if let Some(default_mmat) = &recolor_state.mmat {
				let mut mmat = default_mmat.clone();
				mmat.id.group_id = 0xFFFFFFFF;
				mmat.id.resource_id = 0x00000000;
				mmat.id.instance_id = mmat_instance;
				mmat_instance += 1;
				mmat.name = PascalString::new(&base_name_with_group);
				mmat.family = family.clone();
				mmat.default_material = false;
				resources.push(DecodedResource::Mmat(mmat));
			}

			if let Some(default_txmt) = &recolor_state.txmt {
				let mut txmt = default_txmt.clone();
				txmt.id.group_id = group_id;
				txmt.id.resource_id = hash_crc32(&format!("{}_txmt", base_name));
				txmt.id.instance_id = hash_crc24(&format!("{}_txmt", base_name));
				txmt.block.material_definition = SevenBitString::new(&format!("{}_txmt", base_name_with_group));
				txmt.block.material_description = SevenBitString::new(&base_name_with_group);
				if let Some(txtr_ref) = txmt.block.properties.iter_mut()
					.find(|p| p.name.to_string() == "stdMatBaseTextureName") {
						txtr_ref.value = SevenBitString::new(&format!("##0x{:08x}!{}", group_id, base_name));
					}
				resources.push(DecodedResource::Txmt(txmt));
			}

			if let Some(default_txtr) = &recolor_state.txtr {
				let txtr = Txtr::create_empty(group_id, &base_name_with_group, default_txtr.block.width, default_txtr.block.height, TxtrPurpose::Object);
				resources.push(DecodedResource::Txtr(txtr));
			}
		}
	}

	resources
}

fn get_mmats(resources: &[DecodedResource], use_defaults: bool, subset: &Option<String>) -> Vec<Mmat> {
	let mut subsets_found = Vec::new();
	resources.iter()
		.filter_map(|resource|
			match resource {
				DecodedResource::Mmat(mmat) => {
					let mmat_subset = mmat.subset_name.to_string();
					if (!use_defaults || mmat.default_material) && !subsets_found.contains(&mmat_subset) {
						if subset.as_ref().is_none_or(|s| *s == mmat_subset) {
							subsets_found.push(mmat_subset);
							Some(mmat.clone())
						} else {
							None
						}
					} else {
						None
					}
				}
				_ => None
			})
		.collect::<Vec<Mmat>>()
}

fn get_mmats_in_family(resources: &[DecodedResource], family: &PascalString, subset: &PascalString) -> Vec<Mmat> {
	resources.iter()
		.filter_map(|resource|
			match resource {
				DecodedResource::Mmat(mmat) =>
					if mmat.family == *family && mmat.subset_name == *subset {
						Some(mmat.clone())
					} else {
						None
					},
				_ => None
			})
		.collect::<Vec<Mmat>>()
}

fn get_repo_subsets(resources: &[DecodedResource]) -> Vec<(String, String)> {
	let mut repo_subsets = Vec::new();
	for resource in resources {
		if let DecodedResource::Gmnd(gmnd) = resource {
			for repo_subset in &gmnd.repo_subsets {
				if !repo_subsets.contains(repo_subset) {
					repo_subsets.push(repo_subset.clone());
				}
			}
		}
	}
	repo_subsets
}

fn get_linked_txtr(txmt: &Txmt, resources: &[DecodedResource]) -> Option<Txtr> {
	match txmt.block.properties.iter()
		.find(|p| p.name.to_string() == "stdMatBaseTextureName") {
			Some(txtr_name) => {
				let txtr_name = format!("{}_txtr", dehash_name(&txtr_name.value.to_string()));
				match get_resource_by_name(&txtr_name, TypeId::Txtr, resources) {
					Some(DecodedResource::Txtr(txtr)) => {
						println!("    Found TXTR \"{}\"", txtr_name);
						Some(txtr.clone())
					},
					_ => None
				}
			},
			None => None
	}
}

fn new_family(rng: &mut ThreadRng) -> PascalString {
	PascalString::new(
		&format!("{:08x}-{:04x}-{:04x}-{:04x}-{:04x}{:08x}",
			rng.random::<u32>(),
			rng.random::<u16>(),
			rng.random::<u16>(),
			rng.random::<u16>(),
			rng.random::<u16>(),
			rng.random::<u32>()))
}
