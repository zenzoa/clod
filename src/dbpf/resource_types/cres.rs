use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ Identifier, TypeId, SevenBitString, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::rcol::{ Rcol, RcolBlock };
use crate::dbpf::resource_types::nodes::sg_resource::SGResource;
use crate::dbpf::resource_types::nodes::composition_tree::CompositionTreeNode;
use crate::dbpf::resource_types::nodes::object_graph::ObjectGraphNode;

#[derive(Clone)]
pub struct Cres {
	pub id: Identifier,
	pub data: Vec<u8>
}

impl Cres {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let rcol = Rcol::read(&resource.data)?;
		if !rcol.blocks.is_empty() {
			if let RcolBlock::Cres(_cres_block) = &rcol.blocks[0] {
				let _shpe_ref = (*rcol.links.first().ok_or("SHPE reference not found.")?).clone();
				return Ok(Self {
					id: resource.id.clone(),
					data: resource.data.clone()
				});
			}
		}
		Err("Invalid CRES resource.".into())
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		Ok(self.data.clone())
	}
}

#[derive(Clone)]
pub struct CresBlock {
	version: u32,
	file_name: SevenBitString,
	ogn: ObjectGraphNode,
	chains: Vec<(bool, bool, u32)>,
	purpose: u32
}

impl CresBlock {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let _block_name = PascalString::read::<u8>(cur)?;
		let _block_id = u32::read_le(cur)?;
		let version = u32::read_le(cur)?;

		let typecode = u8::read(cur)?; // 1 = main, 0 = lots only

		if typecode == 1 {
			let file_name = SGResource::read(cur)?.file_name;
			CompositionTreeNode::read(cur)?;
			let ogn = ObjectGraphNode::read(cur)?;

			let num_chains = u32::read_le(cur)?;
			let mut chains = Vec::new();
			for _ in 0..num_chains {
				let is_enabled = u8::read(cur)? == 1;
				let is_dependent = u8::read(cur)? == 1;
				let first_node_location = u32::read_le(cur)?;
				chains.push((is_enabled, is_dependent, first_node_location));
			}

			let _subnode_is_cres = u8::read(cur)? == 1; // expect 0; used for lots only
			let purpose = u32::read_le(cur)?;

			Ok(Self {
				version,
				file_name,
				ogn,
				chains,
				purpose
			})

		} else {
			Err("Invalid CRES block.".into())
		}
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		PascalString::new("cResourceNode").write::<u8>(writer)?;
		(TypeId::Cres as u32).write_le(writer)?;
		self.version.write_le(writer)?;

		1u8.write(writer)?;

		(SGResource { file_name: self.file_name.clone() }).write(writer)?;

		CompositionTreeNode::write(writer)?;

		self.ogn.write(writer)?;

		(self.chains.len() as u32).write_le(writer)?;
		for (is_enabled, is_dependent, first_node_location) in &self.chains {
			if *is_enabled { 1u8.write(writer)?; } else { 0u8.write(writer)?; }
			if *is_dependent { 1u8.write(writer)?; } else { 0u8.write(writer)?; }
			first_node_location.write_le(writer)?;
		}

		0u8.write(writer)?;

		self.purpose.write_le(writer)?;

		Ok(())
	}
}
