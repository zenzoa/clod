use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ Identifier, TypeId, SevenBitString, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::rcol::{ Rcol, RcolBlock };
use crate::dbpf::resource_types::nodes::sg_resource::SGResource;
use crate::dbpf::resource_types::nodes::object_graph::ObjectGraphNode;

#[derive(Clone)]
pub struct Gmnd {
	pub id: Identifier,
	pub block: GmndBlock,
	pub gmdc_ref: Identifier,
	pub data: Vec<u8>
}

impl Gmnd {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let rcol = Rcol::read(&resource.data)?;
		if !rcol.blocks.is_empty() {
			if let RcolBlock::Gmnd(gmnd_block) = &rcol.blocks[0] {
				let gmdc_ref = (*rcol.links.first().ok_or("GMDC reference not found.")?).clone();
				return Ok(Self {
					id: resource.id.clone(),
					block: gmnd_block.clone(),
					gmdc_ref,
					data: resource.data.clone()
				});
			}
		}
		Err("Invalid GMND resource.".into())
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		Ok(self.data.clone())
	}
}

#[derive(Clone)]
pub struct GmndBlock {
	pub version: u32,
	pub file_name: SevenBitString,
	pub ogn: ObjectGraphNode,
	pub redirects_to_geo: bool,
	pub contains_geo: bool,
	pub subblocks: Vec<RcolBlock>
}

impl GmndBlock {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let _block_name = PascalString::read::<u8>(cur)?;
		let _block_id = u32::read_le(cur)?;
		let version = u32::read_le(cur)?;

		let ogn = ObjectGraphNode::read(cur)?;

		let file_name = SGResource::read(cur)?.file_name;

		let redirects_to_geo = version == 11 && u16::read_le(cur)? == 2;
		let contains_geo = (version == 11 || version == 12) && u16::read_le(cur)? == 1;
		let _ = u8::read(cur)?; // expect 1

		let num_subblocks = u32::read_le(cur)?;
		let mut subblocks = Vec::new();
		for _ in 0..num_subblocks {
			let subblock_id = u32::read_le(cur)?;
			subblocks.push(RcolBlock::read(cur, subblock_id)?);
		}

		let gmnd = Self {
			version,
			file_name,
			ogn,
			redirects_to_geo,
			contains_geo,
			subblocks
		};

		Ok(gmnd)
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		PascalString::new("cGeometryNode").write::<u8>(writer)?;
		(TypeId::Gmnd as u32).write_le(writer)?;
		self.version.write_le(writer)?;

		self.ogn.write(writer)?;

		(SGResource { file_name: self.file_name.clone() }).write(writer)?;

		if self.version == 11 {
			if self.redirects_to_geo {
				2u16.write_le(writer)?;
			} else {
				512u16.write_le(writer)?;
			}
		}

		if self.version == 11 || self.version == 12 {
			if self.contains_geo {
				1u16.write_le(writer)?;
			} else {
				256u16.write_le(writer)?;
			}
		}

		1u8.write(writer)?;

		(self.subblocks.len() as u32).write_le(writer)?;
		for subblock in &self.subblocks {
			subblock.write_id(writer)?;
			subblock.write(writer)?;
		}

		Ok(())
	}
}
