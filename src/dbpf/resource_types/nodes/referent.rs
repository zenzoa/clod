use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::SevenBitString;

pub struct ReferentNode;

impl ReferentNode {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<(), Box<dyn Error>> {
		let block_name = SevenBitString::read(cur)?;
		if &block_name.to_string() != "cReferentNode" {
			return Err("Invalid cReferentNode header.".into());
		}

		let _block_id = u32::read_le(cur)?; // expect 0
		let _block_version = u32::read_le(cur)?; // expect 1

		Ok(())
	}

	pub fn write(writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		SevenBitString::new("cReferentNode").write(writer)?;
		0u32.write_le(writer)?;
		1u32.write_le(writer)?;
		Ok(())
	}
}
