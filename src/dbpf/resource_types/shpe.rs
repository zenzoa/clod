use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use regex::Regex;

use crate::crc::{ hash_crc24, hash_crc32 };
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
	pub gmnd_ref: Option<Identifier>
}

impl Shpe {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let rcol = Rcol::read(&resource.data)?;
		if rcol.blocks.len() == 1 {
			if let RcolBlock::Shpe(shpe_block) = &rcol.blocks[0] {
				let gmnd_item = shpe_block.gmnd_items.iter().find(|gmnd_item| {
					gmnd_item.item_type == 0 && !gmnd_item.name.0.is_empty()
				});
				let gmnd_ref = if let Some(gmnd_item) = gmnd_item {
					let re = Regex::new(r"^##0x([0-9,a-f,A-F]+)!(.+)$").unwrap();
					let gmnd_name = gmnd_item.name.to_string();
					if let Some(captures) = re.captures(&gmnd_name) {
						let type_id = u32::from(TypeId::Gmnd);
						let group_id = u32::from_str_radix(&captures[1], 16).unwrap();
						let instance_id = hash_crc24(&captures[2]);
						let resource_id = hash_crc32(&captures[2]);
						Some(Identifier::new(type_id, group_id, resource_id, instance_id))
					} else {
						None
					}
				} else {
					return Err(format!("{} does not define GMND", resource.id).into());
				};
				return Ok(Self {
					id: resource.id.clone(),
					block: shpe_block.clone(),
					gmnd_ref
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
		let mut cur = Cursor::new(Vec::new());
		rcol.write(&mut cur)?;
		Ok(cur.into_inner())
	}
}

#[derive(Clone)]
pub struct ShpeBlock {
	pub version: u32,
	pub file_name: SevenBitString,
	pub ogn: ObjectGraphNode,
	pub lod_values: Vec<u32>,
	pub gmnd_items: Vec<GmndItem>,
	pub materials: Vec<Material>
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

		let mut gmnd_items = Vec::new();
		let num_gmnd_items = u32::read_le(cur)?;
		for _ in 0..num_gmnd_items {
			gmnd_items.push(GmndItem::read(cur, version)?);
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
			gmnd_items,
			materials
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		PascalString::new("cShape").write::<u8>(writer)?;
		u32::from(TypeId::Shpe).write_le(writer)?;
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

		(self.gmnd_items.len() as u32).write_le(writer)?;
		for gmnd_item in &self.gmnd_items {
			gmnd_item.write(writer, self.version)?;
		}

		(self.materials.len() as u32).write_le(writer)?;
		for material in &self.materials {
			material.write(writer)?;
		}

		Ok(())
	}
}

#[derive(Clone)]
pub struct GmndItem {
	pub item_type: u32,
	pub enabled: u8,
	pub use_submesh: u8,
	pub header_link_index: u32,
	pub name: SevenBitString
}

impl GmndItem {
	pub fn read(cur: &mut Cursor<&[u8]>, version: u32) -> Result<Self, Box<dyn Error>> {
		let item_type = u32::read_le(cur)?;
		let enabled = u8::read(cur)?;
		let use_submesh = if version != 8 { u8::read(cur)? } else { 0 };
		let header_link_index = if version != 8 { u32::read_le(cur)? } else { 0 };
		let name = if version == 8 { SevenBitString::read(cur)? } else { SevenBitString::new("") };

		Ok(Self {
			item_type,
			enabled,
			use_submesh,
			header_link_index,
			name
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>, version: u32) -> Result<(), Box<dyn Error>> {
		self.item_type.write_le(writer)?;
		self.enabled.write_le(writer)?;
		if version != 8 {
			self.use_submesh.write_le(writer)?;
			self.header_link_index.write_le(writer)?;
		} else {
			self.name.write(writer)?;
		}
		Ok(())
	}
}

#[derive(Clone)]
pub struct Material {
	pub subset: SevenBitString,
	pub txmt_name: SevenBitString
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
