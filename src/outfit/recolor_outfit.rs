use std::error::Error;
use std::path::PathBuf;
use std::collections::HashMap;

use rand::Rng;

use crate::dbpf::{ Dbpf, Identifier, TypeId, PascalString };
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::text_list::TextList;
use crate::dbpf::resource_types::binx::Binx;
use crate::dbpf::resource_types::gzps::{ Gzps, Part, Age, Gender, Category, Shoe, OutfitFlag, HairTone, SubsetRef };
use crate::dbpf::resource_types::idr::Idr;
use crate::dbpf::resource_types::txmt::Txmt;
use crate::dbpf::resource_types::txtr::{ Txtr, TxtrPurpose };
use crate::crc::hash_crc32;

pub fn recolor_outfit(files: Vec<PathBuf>, output: Option<PathBuf>, multiple: Option<usize>, repo: bool, tooltip: Option<String>, name: Option<String>, ages: Option<Vec<String>>, genders: Option<String>, categories: Option<Vec<String>>, shoe: Option<String>, parts: Option<String>, flags: Option<Vec<String>>, mut sort: Option<i32>, textureless: Option<Vec<String>>, txmts_last: bool) -> Result<(), Box<dyn Error>> {
	let file = &files[0];
	let package = Dbpf::read_from_file(file)?;

	let output_parent = match &output {
		Some(p) => p.parent().ok_or("Invalid output path")?,
		None => file.parent().ok_or("Invalid input path")?
	};

	let output_file_stem = match &output {
		Some(p) => p.file_stem()
			.map(|s| s.to_string_lossy())
			.ok_or("Invalid output path")?
			.to_string(),
		None => file.file_stem()
			.map(|s| s.to_string_lossy()
				.replace("_MESH", "")
				.replace("_Mesh", "")
				.replace("_mesh", ""))
			.ok_or("Invalid output path")?,
	};

	let mut cres_id = None;
	let mut shpe_id = None;
	let mut ui_id = None;
	let mut coll_id = None;
	let mut subsets = Vec::new();

	let mut version = None;
	let mut product = None;
	let mut creator = None;

	let mut name = name.map(|n| PascalString::new(&n));

	let mut ages = ages.map(|age_strings|
		age_strings.iter()
			.flat_map(|a| Age::from_string(a))
			.collect::<Vec<Age>>());

	let mut genders = genders.map(|g| Gender::from_string(&g));

	let mut categories = categories.map(|category_strings|
		category_strings.iter()
			.flat_map(|c| Category::from_string(c))
			.collect::<Vec<Category>>());

	let mut shoe = shoe.map(|s| Shoe::from_string(&s));

	let mut parts = parts.map(|p| Part::from_string(&p));

	let mut flags = flags.map(|flag_strings|
		flag_strings.iter()
			.flat_map(|f| OutfitFlag::from_string(f))
			.collect::<Vec<OutfitFlag>>());

	let mut txmt_ids = Vec::new();

	print!("Reading resources from original outfit...");
	for resource in &package.resources {
		match resource {
			DecodedResource::Cres(cres) => {
				cres_id = Some(cres.id.clone());
			},

			DecodedResource::Shpe(shpe) => {
				shpe_id = Some(shpe.id.clone());
				subsets = shpe.block.materials.iter()
					.map(|m| m.subset.to_string())
					.collect::<Vec<String>>();
			},

			DecodedResource::Idr(idr) => {
				for reference in &idr.references {
					match reference.type_id {
						TypeId::Cres => { cres_id = Some(reference.clone()) }
						TypeId::Shpe => { shpe_id = Some(reference.clone()) }
						TypeId::Ui => { ui_id = Some(reference.clone()) }
						TypeId::Coll => { coll_id = Some(reference.clone()) }
						TypeId::Txmt => { txmt_ids.push(reference.clone()) }
						_ => {}
					}
				}
			},

			DecodedResource::Gzps(gzps) => {
				subsets = gzps.subset_indexes.iter()
					.map(|s| s.subset_name.to_string())
					.collect::<Vec<String>>();
				version = gzps.version;
				product = gzps.product;
				creator = Some(gzps.creator.clone());
				if name.is_none() { name = Some(gzps.name.clone()) }
				if ages.is_none() { ages = Some(gzps.ages.clone()) }
				if genders.is_none() { genders = Some(gzps.genders.clone()) }
				if categories.is_none() { categories = Some(gzps.categories.clone()) }
				if shoe.is_none() { shoe = Some(gzps.shoe) }
				if parts.is_none() { parts = Some(gzps.parts.clone()) }
				if flags.is_none() { flags = Some(gzps.flags.clone()) }
			},

			DecodedResource::Binx(binx) => {
				if sort.is_none() { sort = Some(binx.sort) }
			}

			_ => {}
		}
	}
	println!("DONE");

	let cres_id = cres_id.ok_or("No CRES reference found")?;
	let shpe_id = shpe_id.ok_or("No SHPE reference found")?;
	let ui_id = ui_id.unwrap_or(Identifier { type_id: TypeId::Ui, group_id: 0, resource_id: 0, instance_id: 0 });
	let coll_id = coll_id.unwrap_or(Identifier { type_id: TypeId::Coll, group_id: 0x0FFEFEFE, resource_id: 0x00000000, instance_id: 0x0FFE0080 });
	if subsets.is_empty() {
		return Err("No subsets found".into());
	}

	let txmts_by_subset = get_txmts_by_subset(&txmt_ids, &subsets, &package.resources);

	let name = name.unwrap_or(PascalString::new(&output_file_stem.to_lowercase().replace(" ", "_")));

	let mut ages = ages.ok_or("No age specified, and no GZPS found")?;
	if ages.contains(&Age::Adult) && !ages.contains(&Age::YoungAdult) {
		ages.push(Age::YoungAdult);
	} else if ages.contains(&Age::YoungAdult) && !ages.contains(&Age::Adult) {
		ages.push(Age::Adult);
	}

	let genders = genders.ok_or("No gender specified, and no GZPS found")?;

	let parts = parts.ok_or("No part specified, and no GZPS found")?;

	let categories = match categories {
		Some(c) => c,
		None => {
			println!("No category specified and no GZPS found. Defaulting to 'everyday'.");
			vec![Category::Everyday]
		}
	};

	let shoe = if parts.contains(&Part::Bottom) || parts.contains(&Part::Body) {
		match shoe {
			Some(s) => s,
			None => {
				println!("No shoe specified and no GZPS found. Defaulting to 'normal'.");
				Shoe::Normal
			}
		}
	} else {
		match shoe {
			Some(s) => s,
			None => {
				println!("No shoe specified and no GZPS found. Defaulting to 'none'.");
				Shoe::None
			}
		}
	};

	let flags = match flags {
		Some(f) => f,
		None => {
			println!("No flags specified and no GZPS found. Defaulting to 'notownies'.");
			vec![OutfitFlag::NoTownies]
		}
	};

	let creator = creator.unwrap_or(PascalString::new("00000000-0000-0000-0000-000000000000"));

	let sort = sort.unwrap_or(hash_crc32(&name.to_string()) as i32);

	let repo_files = &files[1..];
	if !repo_files.is_empty() { print!("Getting repository references..."); }
	let repo_txmt_refs = if repo {
		get_repo_txmt_refs(repo_files, &subsets)?
	} else {
		Vec::new()
	};
	if !repo_files.is_empty() { println!("DONE"); }

	let output_count = if repo {
		repo_txmt_refs.len()
	} else {
		multiple.unwrap_or(1)
	};

	println!("Creating recolor packages...");
	let mut rng = rand::rng();
	for i in 0..output_count {
		let guid: u32 = rng.random();

		let output_file_name = if output_count == 1 {
			if output_parent.join(&output_file_stem).with_extension("package").exists() {
				format!("{output_file_stem}_2")
			} else {
				output_file_stem.clone()
			}
		} else {
			format!("{output_file_stem}_{:02}", i+1)
		};
		let output_path = output_parent.join(&output_file_name).with_extension("package");

		let base_tooltip = tooltip.clone().unwrap_or(output_file_stem.clone());
		let tooltip_text = if output_count == 1 {
			base_tooltip
		} else {
			format!("{base_tooltip}_{:02}", i+1)
		};
		let text_list = TextList::from_string(&tooltip_text, guid);

		let gzps = Gzps {
			id: Identifier { type_id: TypeId::Gzps, group_id: guid, resource_id: 0, instance_id: 1 },
			version,
			product,
			ages: ages.clone(),
			genders: genders.clone(),
			species: 1,
			outfit: parts.clone(),
			parts: parts.clone(),
			flags: flags.clone(),
			name: name.clone(),
			creator: creator.clone(),
			family: PascalString::new("00000000-0000-0000-0000-000000000000"),
			genetic: None,
			priority: None,
			outfit_type: PascalString::new("skin"),
			skintone: PascalString::new("00000000-0000-0000-0000-000000000000"),
			hairtone: HairTone::None,
			categories: categories.clone(),
			shoe,
			fitness: 0,
			cres_index: if txmts_last { 4 } else { 0 },
			shpe_index: if txmts_last { 5 } else { 1 },
			subset_indexes: subsets.iter().enumerate().map(|(j, subset)| {
				SubsetRef {
					shpe_index: 0,
					subset_name: PascalString::new(subset),
					txmt_index: if txmts_last { 6 + j as u32 } else { 2 + j as u32 },
				}
			}).collect()
		};

		let (txmts, txtrs) = if !repo {
			make_txmts_txtrs(guid, &format!("{name}_{:02}", i+1), &subsets, &textureless, &txmts_by_subset)
		} else {
			(Vec::new(), Vec::new())
		};

		let references1 = vec![
			ui_id.clone(),
			text_list.id.clone(),
			coll_id.clone(),
			gzps.id.clone()
		];

		let mut references2 = vec![
			cres_id.clone(),
			shpe_id.clone()
		];
		if repo {
			for repo_txmt_ref in &repo_txmt_refs[i] {
				references2.push(repo_txmt_ref.clone());
			}
		} else {
			for txmt in &txmts {
				references2.push(txmt.get_id());
			}
		}

		let references = if txmts_last {
			[references1, references2].concat()
		} else {
			[references2, references1].concat()
		};

		let idr = Idr {
			id: Identifier { type_id: TypeId::Idr, group_id: guid, resource_id: 0, instance_id: 1 },
			references
		};

		let last_index = gzps.max_resource_key();
		let binx = Binx {
			id: Identifier { type_id: TypeId::Binx, group_id: guid, resource_id: 0, instance_id: 1 },
			ui_index: if txmts_last { 0 } else { last_index + 1 },
			text_list_index: if txmts_last { 1 } else { last_index + 2 },
			coll_index: if txmts_last { 2 } else { last_index + 3 },
			gzps_index: if txmts_last { 3 } else { last_index + 4 },
			creator: creator.clone(),
			sort,
			string_index: 1
		};

		let mut resources = vec![
			DecodedResource::Idr(idr),
			DecodedResource::Binx(binx),
			DecodedResource::Gzps(gzps),
			DecodedResource::TextList(text_list)
		];
		resources.extend(txmts);
		resources.extend(txtrs);

		Dbpf::write_package_file(&resources, &output_path)?;
		println!("  {:?}", output_path);
	}
	println!("DONE");

	Ok(())
}

pub fn get_repo_txmt_refs(repo_files: &[PathBuf], subsets: &[String]) -> Result<Vec<Vec<Identifier>>, Box<dyn Error>> {
	let mut repo_txmt_refs = Vec::new();
	for repo_file in repo_files {
		let repo_package = Dbpf::read_from_file(repo_file)?;
		let repo_idrs = repo_package.resources.iter()
			.filter_map(|r| if let DecodedResource::Idr(idr) = r { Some(idr.clone()) } else { None })
			.collect::<Vec<Idr>>();
		for repo_idr in &repo_idrs {
			if let Some(gzps_ref) = repo_idr.refs_by_type(TypeId::Gzps).first() {
				if let Some(repo_gzps) = repo_package.resources.iter()
					.find_map(|r|
						if r.get_id() == *gzps_ref {
							match r {
								DecodedResource::Gzps(gzps) => Some(gzps),
								_ => None
							}
						} else {
							None
						}
					) {
						let txmt_refs = subsets.iter()
							.filter_map(|subset|
								if let Some(subset_index) = repo_gzps.subset_indexes.iter()
									.find(|s| s.subset_name.to_string() == *subset) {
										repo_idr.references.get(subset_index.txmt_index as usize).cloned()
								} else {
									None
								}
							)
							.collect::<Vec<Identifier>>();
						if txmt_refs.len() != subsets.len() {
							return Err(format!("Repo package {} contains mismatched subsets", repo_file.to_string_lossy()).into());
						} else {
							repo_txmt_refs.push(txmt_refs);
						}
				}
			}
		}
	}

	Ok(repo_txmt_refs)
}

pub fn get_txmts_by_subset(txmt_ids: &[Identifier], subsets: &[String], resources: &[DecodedResource]) -> HashMap<String, Txmt> {
	let mut txmts_by_subset = HashMap::new();
	let txmts = txmt_ids.iter()
		.filter_map(|id|
			match id.corresponding_resource(resources) {
				Some(DecodedResource::Txmt(txmt)) => Some(txmt),
				_ => None
			})
		.collect::<Vec<Txmt>>();
	if txmts.len() == txmt_ids.len() && subsets.len() == txmt_ids.len() {
		for (i, subset) in subsets.iter().enumerate() {
			txmts_by_subset.insert(subset.clone(), txmts[i].clone());
		}
	}
	txmts_by_subset
}

pub fn make_txmts_txtrs(guid: u32, name: &str, subsets: &[String], textureless_subsets: &Option<Vec<String>>, txmts_by_subset: &HashMap<String, Txmt>) -> (Vec<DecodedResource>, Vec<DecodedResource>) {
	let mut txmts = Vec::new();
	let mut txtrs = Vec::new();
	for subset in subsets {
		match txmts_by_subset.get(subset) {
			Some(txmt) => {
				let new_txmt = txmt.replace_guid(guid);
				for txtr_name in &new_txmt.txtr_names {
					let base_txtr_name = match txtr_name.to_string().split_once('!') {
						Some((_, base_name)) => base_name.to_string(),
						None => txtr_name.to_string()
					};
					txtrs.push(DecodedResource::Txtr(Txtr::create_empty(
						guid,
						&base_txtr_name,
						1024, 1024,
						TxtrPurpose::Outfit)));
				}
				txmts.push(DecodedResource::Txmt(new_txmt));
			},
			None => {
				let shader = match subset.as_str() {
					"body" | "top" | "bottom" => "SimSkin",
					_ => "SimStandardMaterial"
				};
				if textureless_subsets.clone().is_some_and(|t| t.contains(subset)) {
					txmts.push(DecodedResource::Txmt(Txmt::create_textureless(
						guid,
						&format!("{name}-{subset}"),
						shader)));
				} else {
					txmts.push(DecodedResource::Txmt(Txmt::create_textured(
						&format!("##0x{:08x}!{name}-{subset}", guid),
						guid,
						&format!("{name}-{subset}"),
						shader)));
					txtrs.push(DecodedResource::Txtr(Txtr::create_empty(
						guid,
						&format!("{name}-{subset}"),
						1024, 1024,
						TxtrPurpose::Outfit)))
				}
			}
		}
	}
	(txmts, txtrs)
}
