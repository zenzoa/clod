use std::error::Error;
use std::fs::{ self, DirEntry, read_dir };
use std::path::{ Path, PathBuf };
use std::collections::HashMap;

use crate::dbpf::resource::{DecodedResource, Resource};
use crate::dbpf::resource_types::objd::Objd;
use crate::dbpf::resource_types::text_list::TextList;
use crate::dbpf::resource_types::txtr::TxtrData;
use crate::dbpf::{ Dbpf, TypeId, Identifier };
use crate::dbpf::resource_types::gzps::{Category, Gzps, OutfitFlag};
use crate::dbpf::resource_types::binx::Binx;
use crate::dbpf::resource_types::idr::Idr;
use crate::crc::{ hash_crc24, hash_crc32 };

#[derive(Clone)]
pub struct ExtractedOutfit {
	pub gzps: Gzps,
	pub gzps_idr: Option<Idr>,
	pub binx: Option<Binx>,
	pub binx_idr: Option<Idr>
}

#[derive(Clone)]
pub struct ReplacementOutfit {
	pub title: String,
	pub gzps: Gzps,
	pub cres_id: Identifier,
	pub shpe_id: Identifier,
	pub txmt_ids: Vec<(String, Identifier)>
}

pub fn get_folder_name(path: &Path) -> Result<String, Box<dyn Error>> {
	if !path.is_dir() {
		return Err("Path is not a folder".into())
	}
	let absolute_path = path.canonicalize()?;
	let folder_name = absolute_path.file_name().ok_or("Unable to get folder name")?;
	Ok(folder_name.to_string_lossy().to_string())
}

pub fn get_subfolders(path: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn Error>> {
	Ok(fs::read_dir(path)?
		.filter_map(|entry| entry.ok())
		.filter_map(|entry| if entry.path().is_dir() { Some(entry.path()) } else { None })
		.collect())
}

pub fn get_package_paths_recursively(path: &PathBuf) -> Vec<PathBuf> {
	let mut paths = Vec::new();
	if let Ok(dir_entries) = fs::read_dir(path) {
		for dir_entry in dir_entries {
			if let Ok(dir_entry) = dir_entry {
				let entry_path = dir_entry.path();
				if entry_path.is_file() && entry_path.extension().is_some_and(|e| e == "package") {
					paths.push(entry_path);
				} else if entry_path.is_dir() {
					paths.extend(get_package_paths_recursively(&entry_path));
				}
			}
		}
	}
	paths
}

pub fn get_packages_in_dir(path: &PathBuf) -> Result<Vec<Dbpf>, Box<dyn Error>> {
	let mut dir_entries: Vec<DirEntry> = fs::read_dir(path)?
		.filter_map(|entry| entry.ok()).collect();

	dir_entries.sort_by_key(|entry| entry.file_name().to_string_lossy().into_owned());

	let mut packages = Vec::new();

	for dir_entry in dir_entries {
		let entry_path = dir_entry.path();
		if entry_path.is_file() && entry_path.extension().is_some_and(|e| e == "package") {
			let dbpf = Dbpf::read_from_file(&entry_path)?;
			packages.push(dbpf);
		}
	}

	Ok(packages)
}

pub fn get_resources_in_packages(packages: &[Dbpf]) -> Result<Vec<DecodedResource>, Box<dyn Error>> {
	let mut resources: Vec<DecodedResource> = Vec::new();
	for package in packages {
		for resource in &package.resources {
			if !resources.iter().any(|r| r.get_id() == resource.get_id()) {
				resources.push(resource.clone());
			}
		}
	}
	Ok(resources)
}

pub fn create_folder(output: &Path, folder_name: &str) -> Result<PathBuf, Box<dyn Error>>{
	let folder_path = output.join(folder_name);
	if !folder_path.is_dir() {
		fs::create_dir(&folder_path)?;
	}
	Ok(folder_path)
}

pub fn get_gzps_related_resources(gzps_id: &Identifier, gzps_resources: &[DecodedResource], bin_resources: &[DecodedResource]) -> (Option<Idr>, Option<Idr>, Option<Binx>) {
	let gzps_idr = match gzps_resources.iter()
		.find(|r| r.get_id() == gzps_id.with_type_id(TypeId::Idr)) {
			Some(DecodedResource::Idr(idr)) => Some(idr.clone()),
			Some(_) => None,
			None => None
		};
	let binx_idr = bin_resources.iter()
		.find_map(|r| match r {
			DecodedResource::Idr(idr) =>
				match idr.refs_by_type(TypeId::Gzps).first() {
					Some(g) => if *g == *gzps_id {
							Some(idr.clone())
						} else {
							None
						},
					None => None
				},
			_ => None
		});
	let binx = match &binx_idr {
		Some(binx_idr) => match bin_resources.iter()
			.find(|r| r.get_id() == binx_idr.id.with_type_id(TypeId::Binx)) {
				Some(DecodedResource::Binx(binx)) => Some(binx.clone()),
				Some(_) => None,
				None => None
			}
		None => None
	};

	(gzps_idr, binx_idr, binx)
}

pub fn insert_outfit_cres(cres_id: &Identifier, resources: &[DecodedResource], hashmap: &mut HashMap<Identifier, DecodedResource>) {
	match cres_id.corresponding_resource(resources) {
		Some(cres) => {
			hashmap.insert(cres.get_id(), cres);
		}
		None => {
			if cres_id.group_id != 0x1C0532FA {
				println!("  WARNING: Unable to find {}", cres_id);
			}
		}
	}
}

pub fn insert_outfit_shpe(shpe_id: &Identifier, resources: &[DecodedResource], hashmap: &mut HashMap<Identifier, DecodedResource>) {
	match shpe_id.corresponding_resource(resources) {
		Some(shpe) => {
			if let DecodedResource::Shpe(shpe) = &shpe {
				if let Some(gmnd_ref) = &shpe.gmnd_ref {
					if let Some(gmnd) = gmnd_ref.corresponding_resource(resources) {
						if let DecodedResource::Gmnd(gmnd) = &gmnd {
							if let Some(gmdc) = gmnd.gmdc_ref.corresponding_resource(resources) {
								hashmap.insert(gmdc.get_id(), gmdc);
							}
						}
						hashmap.insert(gmnd.get_id(), gmnd);
					}
				}
			}
			hashmap.insert(shpe.get_id(), shpe);
		}
		None => {
			if shpe_id.group_id != 0x1C0532FA {
				println!("  WARNING: Unable to find {}", shpe_id);
			}
		}
	}
}

pub fn insert_outfit_txmts_txtrs(txmt_ids: &[Identifier], resources: &[DecodedResource], hashmap: &mut HashMap<Identifier, DecodedResource>) {
	for txmt_id in txmt_ids {
		match txmt_id.corresponding_resource(resources) {
			Some(txmt) => {
				if let DecodedResource::Txmt(txmt) = &txmt {
					for txtr_name in &txmt.txtr_names {
						let txtr_name = format!("{}_txtr", txtr_name.to_string().to_lowercase());
						match resources.iter().find(|r| {
								if let DecodedResource::Txtr(txtr) = r {
									let txtr_string = txtr.name.to_string().to_lowercase();
									if (txtr_string.contains('!') && txtr_string == txtr_name) ||
										(format!("##0x{:08x}!{}", txtr.id.group_id, txtr_string) == txtr_name) {
											return true
										}
								}
								false
							}) {
								Some(txtr) => { hashmap.insert(txtr.get_id(), txtr.clone()); }
								None => { println!("  WARNING: Unable to find TXTR {}", txtr_name); }
							}
					}
				}
				hashmap.insert(txmt.get_id(), txmt);
			}
			None => { println!("  WARNING: Unable to find {}", txmt_id); }
		}
	}
}

pub fn get_outfit_related_resources(original_gzps: &Gzps, original_binx: &Option<Binx>, original_idr: &Option<Idr>, replacement_outfit: &ReplacementOutfit, replacement_resources: &[DecodedResource], sort: i32, create_binx: bool) -> HashMap<Identifier, DecodedResource> {
	let mut resources = HashMap::new();

	// Get GZPS
	resources.insert(original_gzps.id.clone(), DecodedResource::Gzps(original_gzps.clone()));

	// Create BINX
	if original_binx.is_none() && create_binx {
		let first_index = original_gzps.max_resource_key() + 1;
		let binx = Binx {
			id: original_gzps.id.with_type_id(TypeId::Binx), //binx_id.clone(),
			ui_index: first_index,
			text_list_index: first_index + 1,
			coll_index: first_index + 2,
			gzps_index: first_index + 3,
			creator: original_gzps.creator.clone(),
			sort,
			string_index: original_binx.clone().map(|b| b.string_index).unwrap_or(1)
		};
		resources.insert(binx.id.clone(), DecodedResource::Binx(binx));
	}

	// Collect references
	let mut binx_references = Vec::new();
	if let Some(ui_ref) = original_idr.as_ref().map(|b| b.refs_by_type(TypeId::Ui).first().cloned()).unwrap_or(None) {
		binx_references.push(ui_ref.clone());
	} else {
		binx_references.push(Identifier{ type_id: TypeId::Ui, group_id: 0, resource_id: 0, instance_id: 0 });
	}
	if let Some(text_list_ref) = original_idr.as_ref().map(|b| b.refs_by_type(TypeId::TextList).first().cloned()).unwrap_or(None) {
		binx_references.push(text_list_ref.clone());
	} else if create_binx {
		let text_list_id = Identifier{ type_id: TypeId::TextList, group_id: original_gzps.id.group_id, resource_id: 0, instance_id: 1 };
		if !resources.contains_key(&text_list_id) {
			let text_list = TextList::create_empty(text_list_id.clone());
			resources.insert(text_list_id.clone(), DecodedResource::TextList(text_list));
		}
		binx_references.push(text_list_id.clone());
	}
	if let Some(coll_ref) = original_idr.as_ref().map(|b| b.refs_by_type(TypeId::Coll).first().cloned()).unwrap_or(None) {
		binx_references.push(coll_ref.clone());
	} else {
		binx_references.push(Identifier{ type_id: TypeId::Coll, group_id: 0x0FFEFEFE, resource_id: 0, instance_id: 0x0FFE0080 });
	}
	binx_references.push(original_gzps.id.clone());

	let mut gzps_references = Vec::new();
	gzps_references.push(replacement_outfit.cres_id.clone());
	gzps_references.push(replacement_outfit.shpe_id.clone());
	gzps_references.extend(replacement_outfit.txmt_ids.iter().map(|t| t.1.clone()));

	// Create 3IDR
	let new_idr = Idr {
		id: original_gzps.id.with_type_id(TypeId::Idr),
		references: if create_binx {
				[gzps_references, binx_references].concat()
			} else {
				gzps_references
			}
	};
	resources.insert(new_idr.id.clone(), DecodedResource::Idr(new_idr));

	// Add CRES, SHPE, GMND, GMDC, TXMTs + TXTRs
	insert_outfit_cres(&replacement_outfit.cres_id, replacement_resources, &mut resources);
	insert_outfit_shpe(&replacement_outfit.shpe_id, replacement_resources, &mut resources);
	let txmt_ids = replacement_outfit.txmt_ids.iter().map(|(_, id)| id.clone()).collect::<Vec<Identifier>>();
	insert_outfit_txmts_txtrs(&txmt_ids, replacement_resources, &mut resources);

	resources
}

pub fn get_objd_related_resources(objd: &Objd, raw_resources: &HashMap<Identifier, Resource>) -> Result<Vec<DecodedResource>, Box<dyn Error>> {
	// Get CTSS
	let ctss_id = Identifier {
		type_id: TypeId::Ctss,
		group_id: objd.id.group_id,
		instance_id: objd.catalog_strings_id as u32,
		resource_id: 0
	};
	let ctss = if let Some(Ok(DecodedResource::Ctss(ctss))) = raw_resources.get(&ctss_id).map(|r| r.decode()) {
		Some(ctss)
	} else {
		None
	};

	// Get CRES resources
	let mut cres_list = HashMap::new();
	let model_names_id = Identifier {
		type_id: TypeId::TextList,
		group_id: objd.id.group_id,
		instance_id: 0x85,
		resource_id: 0
	};
	if let Some(Ok(DecodedResource::TextList(model_names))) = raw_resources.get(&model_names_id).map(|r| r.decode()) {
		for model_name in model_names.get_items_by_language(1) {
			if !model_name.title.is_empty() {
				let cres_name = format!("{}_cres", dehash_name(&model_name.title));
				if let Some(DecodedResource::Cres(cres)) = get_raw_resource_by_name(&cres_name, TypeId::Cres, raw_resources) {
					cres_list.insert(cres.id.clone(), cres);
				}
			}
		}
	}

	// Get SHPE, GMND, and GMDC resources
	let mut shpe_list = HashMap::new();
	let mut gmnd_list = HashMap::new();
	let mut gmdc_list = HashMap::new();
	let mut txmt_names = Vec::new();
	let mut repo_subsets = Vec::new();
	for cres in cres_list.values() {
		for shpe_id in cres.get_shpe_refs() {
			if let Some(Ok(DecodedResource::Shpe(shpe))) = raw_resources.get(&shpe_id).map(|r| r.decode()) {
				for gmnd_item in &shpe.block.gmnd_items {
					if let Some(DecodedResource::Gmnd(gmnd)) = get_raw_resource_by_name(&gmnd_item.name.to_string(), TypeId::Gmnd, raw_resources) {
						if let Some(Ok(DecodedResource::Gmdc(gmdc))) = raw_resources.get(&gmnd.gmdc_ref).map(|r| r.decode()) {
							gmdc_list.insert(gmdc.id.clone(), gmdc);
						}
						for repo_subset in &gmnd.repo_subsets {
							if !repo_subsets.contains(repo_subset) {
								repo_subsets.push(repo_subset.clone());
							}
						}
						gmnd_list.insert(gmnd.id.clone(), gmnd);
					}
				}
				for material in &shpe.block.materials {
					let txmt_name = material.txmt_name.to_string();
					if !txmt_names.contains(&txmt_name) {
						txmt_names.push(txmt_name);
					}
				}
				shpe_list.insert(shpe.id.clone(), shpe);
			}
		}
	}

	// Get MMAT resources
	let mut mmat_list = HashMap::new();
	for (_, raw_mmat) in raw_resources.iter().filter(|(id, _)| id.type_id == TypeId::Mmat) {
		if let Ok(DecodedResource::Mmat(mmat)) = raw_mmat.decode() {
			if mmat.object_guid == objd.guid {
				let txmt_name = mmat.name.to_string();
				for (source_subset, repo_subset_name) in &repo_subsets {
					if *source_subset == mmat.subset_name.to_string() && !txmt_names.contains(&repo_subset_name) {
						txmt_names.push(txmt_name.replace(source_subset, &repo_subset_name));

					}
				}
				if !txmt_names.contains(&txmt_name) {
					txmt_names.push(txmt_name);
				}
				mmat_list.insert(mmat.id.clone(), mmat);
			}
		}
	}

	// Get TXMT, TXTR, and LIFO resources
	let mut txmt_list = HashMap::new();
	let mut txtr_list = HashMap::new();
	let mut lifo_list = HashMap::new();
	for txmt_name in &txmt_names {
		let txmt_name = format!("{}_txmt", dehash_name(txmt_name));
		if let Some(DecodedResource::Txmt(txmt)) = get_raw_resource_by_name(&txmt_name, TypeId::Txmt, raw_resources) {
			for txtr_name in &txmt.txtr_names {
				let txtr_name = format!("{}_txtr", dehash_name(&txtr_name.to_string()));
				if let Some(DecodedResource::Txtr(txtr)) = get_raw_resource_by_name(&txtr_name, TypeId::Txtr, raw_resources) {
					for image_data in txtr.block.image_groups.iter().flatten() {
						if let TxtrData::Lifo(lifo_name) = image_data {
							if let Some(DecodedResource::Lifo(lifo)) = get_raw_resource_by_name(&lifo_name.to_string(), TypeId::Lifo, &raw_resources) {
								lifo_list.insert(lifo.id.clone(), lifo);
							}
						}
					}
					txtr_list.insert(txtr.id.clone(), txtr);
				}
			}
			txmt_list.insert(txmt.id.clone(), txmt);
		}
	}

	let mut resources = vec![DecodedResource::Objd(objd.clone())];
	if let Some(ctss) = ctss {
		resources.push(DecodedResource::Ctss(ctss));
	}
	resources.extend(cres_list.into_values().map(|cres| DecodedResource::Cres(cres)));
	resources.extend(shpe_list.into_values().map(|shpe| DecodedResource::Shpe(shpe)));
	resources.extend(gmnd_list.into_values().map(|gmnd| DecodedResource::Gmnd(gmnd)));
	resources.extend(gmdc_list.into_values().map(|gmdc| DecodedResource::Gmdc(gmdc)));
	resources.extend(mmat_list.into_values().map(|mmat| DecodedResource::Mmat(mmat)));
	resources.extend(txmt_list.into_values().map(|txmt| DecodedResource::Txmt(txmt)));
	resources.extend(txtr_list.into_values().map(|txtr| DecodedResource::Txtr(txtr)));
	resources.extend(lifo_list.into_values().map(|lifo| DecodedResource::Lifo(lifo)));
	Ok(resources)
}

pub fn dehash_name(name: &str) -> String {
	name.split_once('!').map(|s| s.1.to_string()).unwrap_or(name.to_string())
}

pub fn get_resource_by_name(name: &str, type_id: TypeId, resources: &[DecodedResource]) -> Option<DecodedResource> {
	let mut id = Identifier {
		type_id,
		group_id: 0x1C050000,
		instance_id: hash_crc24(name),
		resource_id: hash_crc32(name)
	};
	resources.iter().find(|r| r.get_id() == id).cloned().or_else(|| {
		id.group_id = 0x1C0532FA;
		resources.iter().find(|r| r.get_id() == id).cloned()
	})
}

pub fn get_raw_resource_by_name(name: &str, type_id: TypeId, raw_resources: &HashMap<Identifier, Resource>) -> Option<DecodedResource> {
	let mut id = Identifier {
		type_id,
		group_id: 0x1C050000,
		instance_id: hash_crc24(name),
		resource_id: hash_crc32(name)
	};
	if let Some(Ok(resource)) = raw_resources.get(&id).map(|r| r.decode()) {
		Some(resource)
	} else {
		id.group_id = 0x1C0532FA;
		if let Some(Ok(resource)) = raw_resources.get(&id).map(|r| r.decode()) {
			Some(resource)
		} else {
			None
		}
	}
}

pub fn read_properties_file(path: &Path, quiet: bool) -> Result<(Option<Vec<Category>>, Option<Vec<OutfitFlag>>, bool), Box<dyn Error>> {
	let mut category_overrides = None;
	let mut flag_overrides = None;
	let mut make_unisex = false;
	for entry in (read_dir(&path)?).flatten() {
		if entry.path().is_file() && entry.path().extension().is_some_and(|ext| ext == "properties") {
			if let Some(prop_string) = entry.path().file_stem().map(|s| s.to_string_lossy()) {
				if !quiet { println!("Found .properties file: {prop_string}"); }
				let mut categories = Vec::new();
				if prop_string.contains("everyday") || prop_string.contains("casual") {
					categories.push(Category::Everyday);
				}
				if prop_string.contains("formal") {
					categories.push(Category::Formal);
				}
				if prop_string.contains("underwear") || prop_string.contains("undies") {
					categories.push(Category::Underwear);
				}
				if prop_string.contains("pajamas") || prop_string.contains("sleep") {
					categories.push(Category::Pajamas);
				}
				if prop_string.contains("swim") {
					categories.push(Category::Swim);
				}
				if prop_string.contains("active") || prop_string.contains("athletic") {
					categories.push(Category::Active);
				}
				if prop_string.contains("outerwear") {
					categories.push(Category::Outerwear);
				}
				if prop_string.contains("pregnant") || prop_string.contains("maternity") {
					categories.push(Category::Pregnant);
				}
				if prop_string.contains("all") {
					categories = Category::all();
				}
				category_overrides = Some(categories);

				let mut flags = Vec::new();
				if prop_string.contains("hidden") {
					flags.push(OutfitFlag::Hidden);
				}
				if prop_string.contains("hat") {
					flags.push(OutfitFlag::Hat);
				}
				if prop_string.contains("notownies") {
					flags.push(OutfitFlag::NoTownies);
				}
				if prop_string.contains("noemployees") || prop_string.contains("noworkers") {
					flags.push(OutfitFlag::NoEmployees);
				}
				flag_overrides = Some(flags);

				if prop_string.contains("unisex") {
					make_unisex = true;
				}
			}
		}
	}
	Ok((category_overrides, flag_overrides, make_unisex))
}
