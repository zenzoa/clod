use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ Identifier, TypeId };
use crate::dbpf::header::Header;
use crate::dbpf::resource::Resource;

#[derive(Clone)]
pub struct IndexEntry {
	pub id: Identifier,
	pub resource_offset: u32,
	pub resource_size: u32
}

impl IndexEntry {
	pub fn from_resource(resource: &Resource, offset: u32) -> Self {
		Self {
			id: resource.id.clone(),
			resource_offset: offset,
			resource_size: resource.data.len() as u32
		}
	}

	pub fn read_all(cur: &mut Cursor<&[u8]>, header: &Header) -> Result<(Vec<IndexEntry>, bool), Box<dyn Error>> {
		let mut index_entries = Vec::new();
		let mut has_dir = false;
		for _ in 0..header.index_entry_count as usize {
			let index_entry = Self::read(cur, header.index_minor_version)?;
			if index_entry.id.type_id == TypeId::Dir {
				has_dir = true;
			} else if index_entry.id.type_id != TypeId::Unknown {
				index_entries.push(index_entry);
			}
		}
		Ok((index_entries, has_dir))
	}

	pub fn read(cur: &mut Cursor<&[u8]>, version: u32) -> Result<Self, Box<dyn Error>> {
		let id = Identifier::read(cur, version >= 2)?;
		let resource_offset = u32::read_le(cur)?;
		let resource_size = u32::read_le(cur)?;
		Ok(IndexEntry {
			id,
			resource_offset,
			resource_size
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>, use_tgir: bool) -> Result<(), Box<dyn Error>> {
		self.id.write(writer, use_tgir)?;
		self.resource_offset.write_le(writer)?;
		self.resource_size.write_le(writer)?;
		Ok(())
	}
}
