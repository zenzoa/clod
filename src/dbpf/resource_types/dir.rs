use std::error::Error;
use std::io::Cursor;

use binrw::BinWrite;

use crate::dbpf::Identifier;

#[derive(Clone)]
pub struct Dir {
	pub id: Identifier,
	pub items: Vec<DirItem>
}

#[derive(Clone)]
pub struct DirItem {
	pub id: Identifier,
	pub uncompressed_size: u32
}

impl Dir {
	pub fn new(items: Vec<DirItem>) -> Self {
		Self {
			id: Identifier::new(0xE86B1EEF, 0xE86B1EEF, 0, 0x286B1F03),
			items
		}
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let mut cur = Cursor::new(Vec::new());

		for item in &self.items {
			item.id.write(&mut cur, true)?;
			item.uncompressed_size.write_le(&mut cur)?;
		}

		Ok(cur.into_inner())
	}
}
