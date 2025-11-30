use std::error::Error;
use std::io::{ Cursor, Read, Write };

use refpack::{ CompressionOptions, easy_compress, easy_decompress, format };

use crate::dbpf::{ Identifier, TypeId };
use crate::dbpf::index_entry::IndexEntry;

use crate::dbpf::resource_types::gmdc::Gmdc;
use crate::dbpf::resource_types::gmnd::Gmnd;
use crate::dbpf::resource_types::shpe::Shpe;
use crate::dbpf::resource_types::cres::Cres;

use crate::dbpf::resource_types::txmt::Txmt;
use crate::dbpf::resource_types::txtr::Txtr;

use crate::dbpf::resource_types::gzps::Gzps;
use crate::dbpf::resource_types::idr::Idr;
use crate::dbpf::resource_types::binx::Binx;
use crate::dbpf::resource_types::text_list::TextList;

use crate::dbpf::resource_types::xtol::Xtol;

#[derive(Clone)]
pub enum DecodedResource {
	Gmdc(Gmdc),
	Gmnd(Gmnd),
	Shpe(Shpe),
	Cres(Cres),
	Txmt(Txmt),
	Txtr(Txtr),
	Gzps(Gzps),
	Idr(Idr),
	Binx(Binx),
	Xtol(Xtol),
	TextList(TextList),
	Other(Resource)
}

impl DecodedResource {
	pub fn new(resource: &Resource, title: &str) -> Result<Self, Box<dyn Error>> {
		match resource.id.type_id {
			TypeId::Gmdc => Ok(DecodedResource::Gmdc(Gmdc::new(resource)?)),
			TypeId::Gmnd => Ok(DecodedResource::Gmnd(Gmnd::new(resource)?)),
			TypeId::Shpe => Ok(DecodedResource::Shpe(Shpe::new(resource)?)),
			TypeId::Cres => Ok(DecodedResource::Cres(Cres::new(resource)?)),
			TypeId::Txmt => Ok(DecodedResource::Txmt(Txmt::new(resource)?)),
			TypeId::Txtr => Ok(DecodedResource::Txtr(Txtr::new(resource)?)),
			TypeId::Gzps => Ok(DecodedResource::Gzps(Gzps::new(resource, title)?)),
			TypeId::Idr => Ok(DecodedResource::Idr(Idr::new(resource)?)),
			TypeId::Binx => Ok(DecodedResource::Binx(Binx::new(resource)?)),
			TypeId::Xtol => Ok(DecodedResource::Xtol(Xtol::new(resource)?)),
			TypeId::TextList => Ok(DecodedResource::TextList(TextList::new(resource)?)),
			_ => Ok(DecodedResource::Other(resource.clone()))
		}
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		match self {
			Self::Gmdc(gmdc) => { gmdc.to_bytes() }
			Self::Gmnd(gmnd) => { gmnd.to_bytes() }
			Self::Shpe(shpe) => { shpe.to_bytes() }
			Self::Cres(cres) => { cres.to_bytes() }
			Self::Txmt(txmt) => { txmt.to_bytes() }
			Self::Txtr(txtr) => { txtr.to_bytes() }
			Self::Gzps(gzps) => { gzps.to_bytes() }
			Self::Idr(idr) => { idr.to_bytes() }
			Self::Binx(binx) => { binx.to_bytes() }
			Self::Xtol(xtol) => { xtol.to_bytes() }
			Self::TextList(text_list) => { text_list.to_bytes() }
			Self::Other(resource) => { Ok(resource.data.clone()) }
		}
	}

	pub fn get_id(&self) -> Identifier {
		match self {
			Self::Gmdc(gmdc) => { gmdc.id.clone() }
			Self::Gmnd(gmnd) => { gmnd.id.clone() }
			Self::Shpe(shpe) => { shpe.id.clone() }
			Self::Cres(cres) => { cres.id.clone() }
			Self::Txmt(txmt) => { txmt.id.clone() }
			Self::Txtr(txtr) => { txtr.id.clone() }
			Self::Gzps(gzps) => { gzps.id.clone() }
			Self::Idr(idr) => { idr.id.clone() }
			Self::Binx(binx) => { binx.id.clone() }
			Self::Xtol(xtol) => { xtol.id.clone() }
			Self::TextList(text_list) => { text_list.id.clone() }
			Self::Other(resource) => { resource.id.clone() }
		}
	}

	pub fn to_resource(&self) -> Result<Resource, Box<dyn Error>> {
		Ok(Resource {
			id: self.get_id(),
			data: self.to_bytes()?
		})
	}
}

#[derive(Clone)]
pub struct Resource {
	pub id: Identifier,
	pub data: Vec<u8>
}

impl Resource {
	pub fn read_all(cur: &mut Cursor<&[u8]>, index_entries: &[IndexEntry]) -> Result<Vec<Resource>, Box<dyn Error>> {
		let mut resources = Vec::new();
		for index_entry in index_entries {
			resources.push(Self::read(cur, index_entry)?);
		}
		Ok(resources)
	}

	pub fn read(cur: &mut Cursor<&[u8]>, index_entry: &IndexEntry) -> Result<Self, Box<dyn Error>> {
		cur.set_position(index_entry.resource_offset as u64);
		let mut raw_data = vec![0u8; index_entry.resource_size as usize];
		cur.read_exact(&mut raw_data)?;

		let data = easy_decompress::<format::Maxis>(&raw_data)
			.or_else(|_| easy_decompress::<format::SimEA>(&raw_data))
			// .or_else(|_| easy_decompress::<format::Reference>(&raw_data))
			.unwrap_or(raw_data.clone());

		Ok(Self {
			id: index_entry.id.clone(),
			data
		})
	}

	pub fn decode(&self, title: &str) -> Result<DecodedResource, Box<dyn Error>> {
		DecodedResource::new(self, title)
	}

	pub fn compress(&mut self) -> Result<bool, Box<dyn Error>> {
		let new_data = easy_compress::<format::Maxis>(&self.data, CompressionOptions::Optimal)?;
		if new_data.len() < self.data.len() {
			self.data = new_data;
			Ok(true)
		} else {
			Ok(false)
		}
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		writer.write_all(&self.data)?;
		Ok(())
	}
}
