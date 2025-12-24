use std::error::Error;
use std::io::{ Cursor, Read };

use binrw::{ BinRead, BinWrite };

use regex::Regex;

use crate::crc::{ hash_crc24, hash_crc32 };
use crate::dbpf::{ Identifier, TypeId, SevenBitString, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::rcol::{ Rcol, RcolBlock };
use crate::dbpf::resource_types::nodes::sg_resource::SGResource;

#[derive(Clone)]
pub struct Txtr {
	pub id: Identifier,
	pub block: TxtrBlock,
	pub name: SevenBitString
}

impl Txtr {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let rcol = Rcol::read(&resource.data)?;
		if rcol.blocks.len() == 1 {
			if let RcolBlock::Txtr(txtr_block) = &rcol.blocks[0] {
				return Ok(Self {
					id: resource.id.clone(),
					block: txtr_block.clone(),
					name: txtr_block.file_name.clone()
				});
			}
		}
		Err("Invalid TXTR resource.".into())
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let rcol = Rcol {
			links: Vec::new(),
			blocks: vec![RcolBlock::Txtr(self.block.clone())]
		};
		let mut cur = Cursor::new(Vec::new());
		rcol.write(&mut cur)?;
		Ok(cur.into_inner())
	}

	pub fn replace_guid(&self, new_guid: u32) -> Self {
		let old_guid_str = format!("{:x}", self.id.group_id);
		let new_guid_str = format!("{:x}", new_guid);
		let mut new_txtr = self.clone();
		new_txtr.id.group_id = new_guid;
		new_txtr.block.file_name = new_txtr.block.file_name.replace(&old_guid_str, &new_guid_str);
		let re = Regex::new(r"^##0x([0-9,a-f,A-F]+)!(.+)$").unwrap();
		if let Some(captures) = re.captures(&new_txtr.block.file_name.to_string()) {
			new_txtr.id.resource_id = hash_crc32(&captures[2]);
			new_txtr.id.instance_id = hash_crc24(&captures[2]);
		}
		new_txtr
	}

	pub fn create_empty(guid: u32, title: &str, purpose: TxtrPurpose) -> Self {
		let name = format!("{title}_txtr");
		let resource_id = hash_crc32(&name);
		let instance_id = hash_crc24(&name);
		let id = Identifier::new(u32::from(TypeId::Txtr), guid, resource_id, instance_id);
		let block = TxtrBlock {
			version: 9,
			file_name: SevenBitString::new(&name),
			width: 1024,
			height: 1024,
			format: TxtrFormat::DXT3,
			mipmap_count: 1,
			purpose,
			image_groups: vec![vec![TxtrData::Image(vec![0u8; 1024*1024])]]
		};
		Self {
			id,
			block,
			name: SevenBitString::new(&name)
		}
	}
}

#[derive(Clone)]
pub struct TxtrBlock {
	pub version: u32,
	pub file_name: SevenBitString,
	pub width: u32,
	pub height: u32,
	pub format: TxtrFormat,
	pub mipmap_count: u32,
	pub purpose: TxtrPurpose,
	pub image_groups: Vec<Vec<TxtrData>>
}

impl TxtrBlock {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let _block_name = PascalString::read::<u8>(cur)?;
		let _block_id = u32::read_le(cur)?;
		let version = u32::read_le(cur)?;

		let file_name = SGResource::read(cur)?.file_name;

		let width = u32::read_le(cur)?;
		let height = u32::read_le(cur)?;
		let format = TxtrFormat::from_flag(u32::read_le(cur)?);
		let mipmap_count = u32::read_le(cur)?;
		let purpose = TxtrPurpose::from_flag(f32::read_le(cur)?);

		let image_group_count = u32::read_le(cur)?;
		let _ = u32::read_le(cur)?;

		if version == 9 {
			let _file_name_repeat = SevenBitString::read(cur)?;
		}

		let mut image_groups = Vec::new();
		for _ in 0..image_group_count {
			let mut images = Vec::new();
			let image_count = if version == 9 {
				u32::read_le(cur)?
			} else {
				mipmap_count
			};
			for _ in 0..image_count {
				let data_type = u8::read(cur)?;
				if data_type == 0 {
					let image_data_size = u32::read_le(cur)? as usize;
					let mut image_data = vec![0u8; image_data_size];
					cur.read_exact(&mut image_data)?;
					images.push(TxtrData::Image(image_data));
				} else if data_type == 1 {
					let lifo_name = SevenBitString::read(cur)?;
					images.push(TxtrData::Lifo(lifo_name));
				} else {
					return Err("TXTR resource contains invalid texture".into());
				}
			}
			image_groups.push(images);

			if version == 7 {
				let _creator_id = u32::read_le(cur)?;
			} else {
				let _creator_id = u32::read_le(cur)?;
				let _format_flag = u32::read_le(cur)?;
			}
		}

		Ok(Self {
			version,
			file_name,
			width,
			height,
			format,
			mipmap_count,
			purpose,
			image_groups
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		PascalString::new("cImageData").write::<u8>(writer)?;
		u32::from(TypeId::Txtr).write_le(writer)?;
		self.version.write_le(writer)?;

		(SGResource { file_name: self.file_name.clone() }).write(writer)?;

		self.width.write_le(writer)?;
		self.height.write_le(writer)?;
		self.format.to_flag().write_le(writer)?;
		self.mipmap_count.write_le(writer)?;
		self.purpose.to_flag().write_le(writer)?;

		(self.image_groups.len() as u32).write_le(writer)?;
		0u32.write_le(writer)?;

		if self.version == 9 {
			SevenBitString::new(&self.file_name.to_string().replace("_txtr", "")).write(writer)?;
		}

		for image_group in &self.image_groups {
			if self.version == 9 {
				(image_group.len() as u32).write_le(writer)?;
			}
			for image in image_group {
				match image {
					TxtrData::Image(image_data) => {
						0u8.write(writer)?;
						(image_data.len() as u32).write_le(writer)?;
						image_data.write(writer)?;
					}
					TxtrData::Lifo(lifo_name) => {
						1u8.write(writer)?;
						lifo_name.write(writer)?;
					}
				}
			}
			if self.version == 7 {
				0xff000000u32.write_le(writer)?;
			} else {
				0xffffffffu32.write_le(writer)?;
				0x41200000u32.write_le(writer)?;
			}
		}

		Ok(())
	}
}

#[derive(Clone, Debug)]
pub enum TxtrFormat {
	RawARGB32,
	RawRGB24,
	Alpha,
	DXT1,
	DXT3,
	Grayscale,
	AltARGB32,
	DXT5,
	AltRGB24
}

impl TxtrFormat {
	pub fn from_flag(flag: u32) -> Self {
		match flag {
			1 => Self::RawARGB32,
			2 => Self::RawRGB24,
			3 => Self::Alpha,
			4 => Self::DXT1,
			6 => Self::Grayscale,
			7 => Self::AltARGB32,
			8 => Self::DXT5,
			9 => Self::AltRGB24,
			_ => Self::DXT3,
		}
	}

	pub fn to_flag(&self) -> u32 {
		match self {
			Self::RawARGB32 => 1,
			Self::RawRGB24 => 2,
			Self::Alpha => 3,
			Self::DXT1 => 4,
			Self::DXT3 => 5,
			Self::Grayscale => 6,
			Self::AltARGB32 => 7,
			Self::DXT5 => 8,
			Self::AltRGB24 => 9,
		}
	}
}

#[derive(Clone, Debug)]
pub enum TxtrPurpose {
	Object,
	Outfit,
	Interface
}

impl TxtrPurpose {
	pub fn from_flag(flag: f32) -> Self {
		match flag {
			2.0 => Self::Outfit,
			3.0 => Self::Interface,
			_ => Self::Object
		}
	}

	pub fn to_flag(&self) -> f32 {
		match self {
			Self::Outfit => 1.0,
			Self::Interface => 2.0,
			Self::Object => 3.0
		}
	}
}

#[derive(Clone)]
pub enum TxtrData {
	Lifo(SevenBitString),
	Image(Vec<u8>)
}
