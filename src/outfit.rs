use std::error::Error;

use crate::dbpf::{ TypeId, PascalString };

use crate::dbpf::resource::DecodedResource;

use crate::dbpf::resource_types::gzps::Gzps;
use crate::dbpf::resource_types::idr::Idr;
use crate::dbpf::resource_types::binx::Binx;

use crate::dbpf::resource_types::gmdc::Gmdc;
use crate::dbpf::resource_types::gmnd::Gmnd;
use crate::dbpf::resource_types::shpe::Shpe;
use crate::dbpf::resource_types::cres::Cres;

use crate::dbpf::resource_types::txmt::Txmt;
use crate::dbpf::resource_types::txtr::Txtr;

#[derive(Clone)]
pub struct Outfit {
	pub title: String,

	pub gzps: Gzps,
	pub idr: Idr,
	pub binx: Option<Binx>,

	pub gmdc: Option<Gmdc>,
	pub gmnd: Option<Gmnd>,
	pub shpe: Option<Shpe>,
	pub cres: Option<Cres>,

	pub txmts: Vec<Txmt>,
	pub txtrs: Vec<Txtr>
}

impl Outfit {
	pub fn from_resources(gzps: Gzps, resources: &[DecodedResource], ignore_missing: bool) -> Result<Self, Box<dyn Error>> {
		// find 3IDR
		let idr = resources.iter().find_map(|res| -> Option<Idr> {
			if let DecodedResource::Idr(idr) = res {
				if idr.id.group_id == gzps.id.group_id
					&& idr.id.instance_id == gzps.id.instance_id
					&& idr.id.resource_id == gzps.id.resource_id {
						return Some(idr.clone());
				}
			}
			None
		}).ok_or(format!("Missing 3IDR for {}", gzps.id))?.clone();

		// find SHPE
		let shpe = if let Some(shpe_ref) = &idr.shpe_ref {
			resources.iter().find_map(|res| -> Option<Shpe> {
				if let DecodedResource::Shpe(shpe) = res {
					if shpe.id == *shpe_ref {
						return Some(shpe.clone());
					}
				}
				None
			})
		} else {
			return Err("3IDR does not define SHPE".into())
		};

		if !ignore_missing && shpe.is_none() {
			return Err(format!("Missing {}", idr.shpe_ref.unwrap()).into());
		}

		// find CRES
		let cres = if let Some(cres_ref) = &idr.cres_ref {
			resources.iter().find_map(|res| -> Option<Cres> {
				if let DecodedResource::Cres(cres) = res {
					if cres.id == *cres_ref {
						return Some(cres.clone());
					}
				}
				None
			})
		} else {
			return Err("3IDR does not define CRES".into())
		};

		if !ignore_missing && cres.is_none() {
			return Err(format!("Missing {}", idr.cres_ref.unwrap()).into());
		}

		// find GMND
		let gmnd = if let Some(shpe) = &shpe {
			resources.iter().find_map(|res| -> Option<Gmnd> {
				if let DecodedResource::Gmnd(gmnd) = res {
					if gmnd.id == shpe.gmnd_ref {
						return Some(gmnd.clone());
					}
				}
				None
			})
		} else {
			None
		};

		if !ignore_missing && shpe.is_some() && gmnd.is_none() {
			return Err(format!("Missing {}", shpe.unwrap().gmnd_ref).into());
		}

		// find GMDC
		let gmdc = if let Some(gmnd) = &gmnd {
			resources.iter().find_map(|res| -> Option<Gmdc> {
				if let DecodedResource::Gmdc(gmdc) = res {
					if gmdc.id == gmnd.gmdc_ref {
						return Some(gmdc.clone());
					}
				}
				None
			})
		} else {
			None
		};

		if !ignore_missing && gmnd.is_some() && gmdc.is_none() {
			return Err(format!("Missing {}", gmnd.unwrap().gmdc_ref).into());
		}

		// find TXMTs
		let mut txmts = Vec::new();
		for txmt_ref in &idr.txmt_refs {
			if let Some(txmt) =
				resources.iter().find_map(|res| -> Option<Txmt> {
					if let DecodedResource::Txmt(txmt) = res {
						if txmt.id == *txmt_ref {
							return Some(txmt.clone());
						}
					}
					None
				}) {
					txmts.push(txmt);
				} else if !ignore_missing {
					return Err(format!("Missing {}", txmt_ref).into());
				}
		}

		// find TXTRs
		let mut txtrs = Vec::new();
		let all_txtrs: Vec<Txtr> = resources
			.iter()
			.filter_map(|r| if let DecodedResource::Txtr(txtr) = r { Some(txtr.clone()) } else { None })
			.collect();
		for txmt in &txmts {
			let txtr_names: Vec<String> = txmt.txtr_names.iter().map(|s| format!("{}_txtr", s)).collect();
			for txtr in &all_txtrs {
				if txtr_names.contains(&txtr.name.to_string()) {
					txtrs.push(txtr.clone());
				}
			}
		}

		Ok(Self {
			title: gzps.title.clone(),

			gzps,
			idr,
			binx: None,

			gmdc,
			gmnd,
			shpe,
			cres,

			txmts,
			txtrs,
		})
	}

	pub fn get_resources(&self) ->Vec<DecodedResource> {
		let mut resources = vec![
			DecodedResource::Gzps(self.gzps.clone()),
			DecodedResource::Idr(self.idr.clone())
		];

		if let Some(binx) = &self.binx {
			resources.push(DecodedResource::Binx(binx.clone()));
		}

		if let Some(gmdc) = &self.gmdc {
			resources.push(DecodedResource::Gmdc(gmdc.clone()));
		}

		if let Some(gmnd) = &self.gmnd {
			resources.push(DecodedResource::Gmnd(gmnd.clone()));
		}

		if let Some(shpe) = &self.shpe {
			resources.push(DecodedResource::Shpe(shpe.clone()));
		}

		if let Some(cres) = &self.cres {
			resources.push(DecodedResource::Cres(cres.clone()));
		}

		for txmt in &self.txmts {
			resources.push(DecodedResource::Txmt(txmt.clone()));
		}

		for txtr in &self.txtrs {
			resources.push(DecodedResource::Txtr(txtr.clone()));
		}

		resources
	}

	pub fn generate_binx(&mut self) {
		let mut id = self.idr.id.clone();
		id.type_id = TypeId::Binx;
		let key = self.gzps.max_resource_key();
		self.binx = Some(Binx {
			id,
			icon_idx: key + 1,
			stringset_idx: key + 2,
			bin_idx: key + 3,
			object_idx: key + 4,
			creator_id: PascalString::new("00000000-0000-0000-0000-000000000000"),
			sort_index: 0,
			string_index: 1
		});
	}
}
