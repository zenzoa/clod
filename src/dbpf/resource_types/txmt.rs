use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::crc::{ hash_crc24, hash_crc32 };
use crate::dbpf::{ Identifier, TypeId, SevenBitString, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::rcol::{ Rcol, RcolBlock };
use crate::dbpf::resource_types::nodes::sg_resource::SGResource;

#[derive(Clone)]
pub struct Txmt {
	pub id: Identifier,
	pub block: TxmtBlock,
	pub txtr_names: Vec<SevenBitString>
}

impl Txmt {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let rcol = Rcol::read(&resource.data)?;
		if rcol.blocks.len() == 1 {
			if let RcolBlock::Txmt(txmt_block) = &rcol.blocks[0] {
				let txtr_names: Vec<SevenBitString> = txmt_block.properties
					.iter()
					.filter_map(|prop|
						if &prop.name.to_string() == "stdMatBaseTextureName" ||
							&prop.name.to_string() == "stdMatNormalMapTextureName" {
								Some(prop.value.clone())
						} else {
							None
						}).collect();
				return Ok(Self {
					id: resource.id.clone(),
					block: txmt_block.clone(),
					txtr_names
				})
			}
		}
		Err("Invalid TXMT resource.".into())
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let rcol = Rcol {
			links: Vec::new(),
			blocks: vec![RcolBlock::Txmt(self.block.clone())]
		};
		let mut cur = Cursor::new(Vec::new());
		rcol.write(&mut cur)?;
		Ok(cur.into_inner())
	}

	pub fn create_textureless(group_id: u32, title: &str, material: &str) -> Self {
		let title_txmt = format!("{title}_txmt");
		let id = Identifier {
			type_id: TypeId::Txmt,
			group_id,
			instance_id: hash_crc24(&title_txmt),
			resource_id: hash_crc32(&title_txmt)
		};
		Self {
			id,
			block: TxmtBlock {
				version: 11,
				material_definition: SevenBitString::new(&title_txmt),
				material_description: SevenBitString::new(&format!("##0x{:08x}!{}", group_id, title)),
				material_type: SevenBitString::new(material),
				properties: vec![
					TxmtProperty::new("deprecatedStdMatInvDiffuseCoeffMultiplier", "1"),
					TxmtProperty::new("reflectivity", "0.5"),
					TxmtProperty::new("stdMatDiffCoef", "1,1,1,1"),
					TxmtProperty::new("stdMatSpecCoef", "0.099,0.099,0.099"),
					TxmtProperty::new("stdMatSpecPower", "2"),
				]
			},
			txtr_names: Vec::new(),
		}
	}

	pub fn create_textured(texture_name: &str, group_id: u32, title: &str, material: &str) -> Self {
		let mut txmt = Self::create_textureless(group_id, title, material);
		txmt.block.properties[1].value = SevenBitString::new("0");
		txmt.block.properties.push(TxmtProperty::new("stdMatBaseTextureName", texture_name));
		txmt.txtr_names.push(SevenBitString::new(texture_name));
		txmt
	}

	pub fn replace_guid(&self, new_guid: u32) -> Self {
		let old_guid_str = format!("{:08x}", self.id.group_id);
		let new_guid_str = format!("{:08x}", new_guid);
		let mut new_txmt = self.clone();
		new_txmt.id.group_id = new_guid;
		new_txmt.block.material_definition = new_txmt.block.material_definition.replace(&old_guid_str, &new_guid_str);
		new_txmt.block.material_description = new_txmt.block.material_description.replace(&old_guid_str, &new_guid_str);
		for prop in new_txmt.block.properties.iter_mut() {
			match prop.name.to_string().as_str() {
				"stdMatBaseTextureName" | "stdMatNormalMapTextureName" | "stdMatEnvCubeTextureName" => {
					prop.value = prop.value.replace(&old_guid_str, &new_guid_str);
				}
				_ => {}
			}
		}
		new_txmt
	}
}

#[derive(Clone)]
pub struct TxmtProperty {
	pub name: SevenBitString,
	pub value: SevenBitString
}

impl TxmtProperty {
	pub fn new(name: &str, value: &str) -> Self {
		Self {
			name: SevenBitString::new(name),
			value: SevenBitString::new(value)
		}
	}
}

#[derive(Clone)]
pub struct TxmtBlock {
	pub version: u32,
	pub material_definition: SevenBitString,
	pub material_description: SevenBitString,
	pub material_type: SevenBitString,
	pub properties: Vec<TxmtProperty>
}

impl TxmtBlock {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let _block_name = PascalString::read::<u8>(cur)?;
		let _block_id = u32::read_le(cur)?;
		let version = u32::read_le(cur)?;

		let material_definition = SGResource::read(cur)?.file_name;
		let material_description = SevenBitString::read(cur)?;
		let material_type = SevenBitString::read(cur)?;

		let num_props = u32::read_le(cur)?;
		let mut properties = Vec::new();
		for _ in 0..num_props {
			properties.push(TxmtProperty {
				name: SevenBitString::read(cur)?,
				value: SevenBitString::read(cur)?
			});
		}

		Ok(Self {
			version,
			material_definition,
			material_description,
			material_type,
			properties
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		PascalString::new("cMaterialDefinition").write::<u8>(writer)?;
		u32::from(TypeId::Txmt).write_le(writer)?;
		self.version.write_le(writer)?;

		(SGResource { file_name: self.material_definition.clone() }).write(writer)?;

		self.material_description.write(writer)?;
		self.material_type.write(writer)?;

		let mut texture_names = Vec::new();
		(self.properties.len() as u32).write_le(writer)?;
		for prop in &self.properties {
			prop.name.write(writer)?;
			prop.value.write(writer)?;
			let prop_name = prop.name.to_string();
			if prop_name == "stdMatBaseTextureName" ||
				prop_name == "stdMatNormalMapTextureName" ||
				prop_name == "stdMatEnvCubeTextureName" {
					texture_names.push(prop.value.clone());
			}
		}

		if self.version > 8 {
			(texture_names.len() as u32).write_le(writer)?;
			for texture_name in &texture_names{
				texture_name.write(writer)?;
			}
		}

		Ok(())
	}
}
