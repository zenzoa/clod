use std::error::Error;
use std::io::{ Cursor, Read };

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ Identifier, TypeId, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::rcol::{ Rcol, RcolBlock };

#[derive(Clone)]
pub struct Gmdc {
	pub id: Identifier,
	// pub block: GmdcBlock,
	pub data: Vec<u8>
}

impl Gmdc {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let rcol = Rcol::read(&resource.data)?;
		if rcol.blocks.len() == 1 {
			if let RcolBlock::Gmdc(_gmdc_block) = &rcol.blocks[0] {
				return Ok(Self {
					id: resource.id.clone(),
					// block: gmdc_block.clone(),
					data: resource.data.clone()
				});
			}
		}
		Err("Invalid GMDC resource.".into())
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		// let rcol = Rcol {
		// 	links: Vec::new(),
		// 	blocks: vec![RcolBlock::Gmdc(self.block.clone())]
		// };
		// let bytes: Vec<u8> = Vec::new();
		// let mut cur = Cursor::new(bytes);
		// rcol.write(&mut cur)?;
		// Ok(cur.into_inner())

		Ok(self.data.clone())
	}
}

#[derive(Clone)]
pub struct GmdcBlock {
	version: u32,
	remaining_data: Vec<u8>
}

impl GmdcBlock {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let _block_name = PascalString::read::<u8>(cur)?;
		let _block_id = u32::read_le(cur)?;
		let version = u32::read_le(cur)?;

		let mut remaining_data: Vec<u8> = Vec::new();
		cur.read_to_end(&mut remaining_data)?;

		Ok(Self {
			version,
			remaining_data
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		PascalString::new("cGeometryDataContainer").write::<u8>(writer)?;
		(TypeId::Gmdc as u32).write_le(writer)?;
		self.version.write_le(writer)?;

		self.remaining_data.write(writer)?;

		Ok(())
	}
}
