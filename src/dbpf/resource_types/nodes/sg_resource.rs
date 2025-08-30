use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::SevenBitString;

pub struct SGResource {
	pub file_name: SevenBitString
}

impl SGResource {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let block_name = SevenBitString::read(cur)?;
		if &block_name.to_string() != "cSGResource" {
			return Err(format!("Invalid cSGResource header.").into());
		}

		let _block_id = u32::read_le(cur)?; // expect 0
		let _block_version = u32::read_le(cur)?; // expect 2

		let file_name = SevenBitString::read(cur)?;

		Ok(Self {
			file_name
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		SevenBitString::new("cSGResource").write(writer)?;
		0u32.write_le(writer)?;
		2u32.write_le(writer)?;
		self.file_name.write(writer)?;
		Ok(())
	}
}
