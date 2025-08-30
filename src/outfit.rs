use std::error::Error;

use crate::dbpf::resource::DecodedResource;

use crate::dbpf::resource_types::gzps::Gzps;
use crate::dbpf::resource_types::idr::Idr;

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

	pub gmdc: Gmdc,
	pub gmnd: Gmnd,
	pub shpe: Shpe,
	pub cres: Cres,

	pub txmts: Vec<Txmt>,
	pub txtrs: Vec<Txtr>
}

impl Outfit {
	pub fn from_resources(gzps: Gzps, resources: &[DecodedResource]) -> Result<Self, Box<dyn Error>> {
		let idr = resources.iter()
			.find_map(|res| -> Option<Idr> {
				if let DecodedResource::Idr(idr) = res {
					if idr.id.group_id == gzps.id.group_id
						&& idr.id.instance_id == gzps.id.instance_id
						&& idr.id.resource_id == gzps.id.resource_id {
							return Some(idr.clone());
					}
				}
				None
			})
			.ok_or(format!("Missing 3IDR for {}", gzps.id))?.clone();

		let shpe = if let Some(shpe_ref) = &idr.shpe_ref {
			resources.iter().find_map(|res| -> Option<Shpe> {
				if let DecodedResource::Shpe(shpe) = res {
					if shpe.id == *shpe_ref {
						return Some(shpe.clone());
					}
				}
				None
			}).ok_or(format!("Missing {}", shpe_ref))?.clone()
		} else {
			return Err("3IDR does not define SHPE".into())
		};

		let cres = if let Some(cres_ref) = &idr.cres_ref {
			resources.iter().find_map(|res| -> Option<Cres> {
				if let DecodedResource::Cres(cres) = res {
					if cres.id == *cres_ref {
						return Some(cres.clone());
					}
				}
				None
			}).ok_or(format!("Missing {}", cres_ref))?.clone()
		} else {
			return Err("3IDR does not define CRES".into())
		};

		let gmnd = resources.iter()
			.find_map(|res| -> Option<Gmnd> {
				if let DecodedResource::Gmnd(gmnd) = res {
					if gmnd.block.file_name.to_string() == shpe.gmnd_name.to_string()[13..] {
						return Some(gmnd.clone());
					}
				}
				None
			}).ok_or(format!("Missing GMND \"{}\"", shpe.gmnd_name))?.clone();

		let gmdc = resources.iter()
			.find_map(|res| -> Option<Gmdc> {
				if let DecodedResource::Gmdc(gmdc) = res {
					if gmdc.id == gmnd.gmdc_ref {
						return Some(gmdc.clone());
					}
				}
				None
			}).ok_or(format!("Missing {}", gmnd.gmdc_ref))?.clone();

		let mut txmts = Vec::new();
		for txmt_ref in &idr.txmt_refs {
			let txmt = resources.iter()
				.find_map(|res| -> Option<Txmt> {
					if let DecodedResource::Txmt(txmt) = res {
						if txmt.id == *txmt_ref {
							return Some(txmt.clone());
						}
					}
					None
				}).ok_or(format!("Missing {}", txmt_ref))?.clone();
			txmts.push(txmt);
		}

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

			gmdc,
			gmnd,
			shpe,
			cres,

			txmts,
			txtrs,
		})
	}

	pub fn get_resources(&self) ->Vec<DecodedResource> {
		let mut resources = Vec::new();

		resources.push(DecodedResource::Gzps(self.gzps.clone()));
		resources.push(DecodedResource::Idr(self.idr.clone()));

		resources.push(DecodedResource::Gmdc(self.gmdc.clone()));
		resources.push(DecodedResource::Gmnd(self.gmnd.clone()));
		resources.push(DecodedResource::Shpe(self.shpe.clone()));
		resources.push(DecodedResource::Cres(self.cres.clone()));

		for txmt in &self.txmts {
			resources.push(DecodedResource::Txmt(txmt.clone()));
		}

		for txtr in &self.txtrs {
			resources.push(DecodedResource::Txtr(txtr.clone()));
		}

		resources
	}
}
