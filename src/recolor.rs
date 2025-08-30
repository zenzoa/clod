use std::error::Error;
use crate::dbpf::resource::DecodedResource;

use crate::dbpf::resource_types::gzps::Gzps;
use crate::dbpf::resource_types::idr::Idr;
use crate::dbpf::resource_types::txmt::Txmt;
use crate::dbpf::resource_types::txtr::Txtr;

pub struct Recolor {
	gzps: Gzps,
	idr: Idr,
	txmts: Vec<Txmt>,
	txtrs: Vec<Txtr>
}

impl Recolor {

	pub fn get_resources(&self) -> Vec<DecodedResource> {
		let txmt_resources: Vec<DecodedResource> = self.txmts
			.iter()
			.map(|txmt| DecodedResource::Txmt(txmt.clone()))
			.collect();

		let txtr_resources: Vec<DecodedResource> = self.txtrs
			.iter()
			.map(|txtr| DecodedResource::Txtr(txtr.clone()))
			.collect();

		[txmt_resources, txtr_resources].concat()
	}
}
