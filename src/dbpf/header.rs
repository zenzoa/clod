use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

#[derive(Clone)]
pub struct Header {
	pub major_version: u32,
	pub minor_version: u32,
	pub index_major_version: u32,
	pub index_minor_version: u32,
	pub index_entry_count: u32,
	pub index_offset: u32,
	pub index_size: u32
}

impl Default for Header {
    fn default() -> Self {
		Self {
			major_version: 1,
			minor_version: 1,
			index_major_version: 7,
			index_minor_version: 2,
			index_entry_count: 0,
			index_offset: 0,
			index_size: 0
		}
	}
}

impl Header {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let label = <[u8; 4]>::read_le(cur)?;
		if String::from_utf8(label.to_vec())? != "DBPF" {
			return Err("Not a DBPF file.".into());
		}

		let major_version = u32::read_le(cur)?;
		let minor_version = u32::read_le(cur)?;
		if major_version > 1 {
			return Err("Not a Sims 2 DBPF file.".into());
		}

		let _unknown1 = u32::read_le(cur)?;
		let _unknown2 = u32::read_le(cur)?;
		let _unknown3 = u32::read_le(cur)?;
		let _date_created = u32::read_le(cur)?;
		let _date_modified = u32::read_le(cur)?;

		let index_major_version = u32::read_le(cur)?;
		let index_entry_count = u32::read_le(cur)?;
		let index_offset = u32::read_le(cur)?;
		let index_size = u32::read_le(cur)?;

		let _hole_entry_count = u32::read_le(cur)?;
		let _hole_offset = u32::read_le(cur)?;
		let _hole_size = u32::read_le(cur)?;

		let index_minor_version = if minor_version >= 1 { u32::read_le(cur)? } else { 0 };

		let _unknown4 = u32::read_le(cur)?;
		let _unknown5 = u32::read_le(cur)?;

		Ok(Header {
			major_version,
			minor_version,
			index_major_version,
			index_entry_count,
			index_offset,
			index_size,
			index_minor_version
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		"DBPF".as_bytes().write(writer)?;
		self.major_version.write_le(writer)?;
		self.minor_version.write_le(writer)?;
		[0u8; 12].write(writer)?;
		0u32.write_le(writer)?; // date_created
		0u32.write_le(writer)?; // date_modified
		self.index_major_version.write_le(writer)?;
		self.index_entry_count.write_le(writer)?;
		self.index_offset.write_le(writer)?;
		self.index_size.write_le(writer)?;
		0u32.write_le(writer)?; // hole_entry_count
		0u32.write_le(writer)?; // hole_offset
		0u32.write_le(writer)?; // hole_size
		if self.minor_version >= 1 {
			self.index_minor_version.write_le(writer)?;
		}
		[0u8; 32].write(writer)?;
		Ok(())
	}
}
