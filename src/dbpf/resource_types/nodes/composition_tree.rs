use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::SevenBitString;

pub struct CompositionTreeNode;

impl CompositionTreeNode {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<(), Box<dyn Error>> {
		let block_name = SevenBitString::read(cur)?;
		if &block_name.to_string() != "cCompositionTreeNode" {
			return Err("Invalid cCompositionTreeNode header.".into());
		}

		let _block_id = u32::read_le(cur)?;
		let _block_version = u32::read_le(cur)?;

		Ok(())
	}

	pub fn write(writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		SevenBitString::new("cCompositionTreeNode").write(writer)?;
		0u32.write_le(writer)?;
		11u32.write_le(writer)?;
		Ok(())
	}
}
