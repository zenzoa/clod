use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ Identifier, TypeId, SevenBitString, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::rcol::{ Rcol, RcolBlock };
use crate::dbpf::resource_types::nodes::sg_resource::SGResource;
use crate::dbpf::resource_types::nodes::referent::ReferentNode;
use crate::dbpf::resource_types::nodes::object_graph::ObjectGraphNode;

#[derive(Clone)]
pub struct Shpe {
	pub id: Identifier,
	pub block: ShpeBlock,
	pub gmnd_name: SevenBitString
}

impl Shpe {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let rcol = Rcol::read(&resource.data)?;
		if rcol.blocks.len() == 1 {
			if let RcolBlock::Shpe(shpe_block) = &rcol.blocks[0] {
				let full_detail_lod = shpe_block.lods.iter().find(|lod| {
					lod.lod_type == 0 && !lod.gmnd_name.0.is_empty()
				});
				let gmnd_name = if let Some(lod) = full_detail_lod {
					lod.gmnd_name.clone()
				} else {
					return Err("SHPE does not define GMND".into());
				};
				return Ok(Self {
					id: resource.id.clone(),
					block: shpe_block.clone(),
					gmnd_name
				});
			}
		}
		Err("Invalid SHPE resource.".into())
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let rcol = Rcol {
			links: Vec::new(),
			blocks: vec![RcolBlock::Shpe(self.block.clone())]
		};
		let bytes: Vec<u8> = Vec::new();
		let mut cur = Cursor::new(bytes);
		rcol.write(&mut cur)?;
		Ok(cur.into_inner())
	}
}

#[derive(Clone)]
pub struct ShpeBlock {
	version: u32,
	file_name: SevenBitString,
	ogn: ObjectGraphNode,
	lod_values: Vec<u32>,
	lods: Vec<Lod>,
	materials: Vec<Material>
}

impl ShpeBlock {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let _block_name = PascalString::read::<u8>(cur)?;
		let _block_id = u32::read_le(cur)?;
		let version = u32::read_le(cur)?;

		let file_name = SGResource::read(cur)?.file_name;

		ReferentNode::read(cur)?;

		let ogn = ObjectGraphNode::read(cur)?;

		let mut lod_values = Vec::new();
		if version > 6 {
			let num_lods = u32::read_le(cur)?;
			for _ in 0..num_lods {
				lod_values.push(u32::read_le(cur)?);
			}
		}

		let mut lods = Vec::new();
		let num_lods = u32::read_le(cur)?;
		for _ in 0..num_lods {
			lods.push(Lod::read(cur, version)?);
		}

		let mut materials = Vec::new();
		let num_materials = u32::read_le(cur)?;
		for _ in 0..num_materials {
			materials.push(Material::read(cur)?);
		}

		Ok(Self {
			version,
			file_name,
			ogn,
			lod_values,
			lods,
			materials
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		PascalString::new("cShape").write::<u8>(writer)?;
		(TypeId::Shpe as u32).write_le(writer)?;
		self.version.write_le(writer)?;

		(SGResource { file_name: self.file_name.clone() }).write(writer)?;

		ReferentNode::write(writer)?;

		self.ogn.write(writer)?;

		if self.version > 6 {
			(self.lod_values.len() as u32).write_le(writer)?;
			for lod_value in &self.lod_values {
				lod_value.write_le(writer)?;
			}
		}

		(self.lods.len() as u32).write_le(writer)?;
		for lod in &self.lods {
			lod.write(writer, self.version)?;
		}

		(self.materials.len() as u32).write_le(writer)?;
		for material in &self.materials {
			material.write(writer)?;
		}

		Ok(())
	}
}

#[derive(Clone)]
struct Lod {
	lod_type: u32,
	enabled: u8,
	use_submesh: u8,
	header_link_index: u32,
	gmnd_name: SevenBitString
}

impl Lod {
	pub fn read(cur: &mut Cursor<&[u8]>, version: u32) -> Result<Self, Box<dyn Error>> {
		let lod_type = u32::read_le(cur)?;
		let enabled = u8::read(cur)?;
		let use_submesh = if version != 8 { u8::read(cur)? } else { 0 };
		let header_link_index = if version != 8 { u32::read_le(cur)? } else { 0 };
		let gmnd_name = if version == 8 { SevenBitString::read(cur)? } else { SevenBitString::new("") };

		Ok(Self {
			lod_type,
			enabled,
			use_submesh,
			header_link_index,
			gmnd_name
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>, version: u32) -> Result<(), Box<dyn Error>> {
		self.lod_type.write_le(writer)?;
		self.enabled.write_le(writer)?;
		if version != 8 {
			self.use_submesh.write_le(writer)?;
			self.header_link_index.write_le(writer)?;
		} else {
			self.gmnd_name.write(writer)?;
		}
		Ok(())
	}
}

#[derive(Clone)]
struct Material {
	subset: SevenBitString,
	txmt_name: SevenBitString
}

impl Material {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let subset = SevenBitString::read(cur)?;
		let txmt_name = SevenBitString::read(cur)?;
		let _lod_type = u32::read_le(cur)?; // always 0
		let _enabled = u8::read_le(cur)?; // always 0
		let _index = u32::read_le(cur)?; // always 0

		Ok(Self {
			subset,
			txmt_name
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		self.subset.write(writer)?;
		self.txmt_name.write(writer)?;
		0u32.write_le(writer)?;
		0u8.write_le(writer)?;
		0u32.write_le(writer)?;
		Ok(())
	}
}
