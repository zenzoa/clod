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
			new_txtr.id.instance_id = hash_crc24(&captures[2]);
			new_txtr.id.resource_id = hash_crc32(&captures[2]);
		}
		new_txtr
	}
}

#[derive(Clone)]
pub struct TxtrBlock {
	pub version: u32,
	pub file_name: SevenBitString,
	pub remaining_data: Vec<u8>
}

impl TxtrBlock {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let _block_name = PascalString::read::<u8>(cur)?;
		let _block_id = u32::read_le(cur)?;
		let version = u32::read_le(cur)?;

		let file_name = SGResource::read(cur)?.file_name;

		let mut remaining_data: Vec<u8> = Vec::new();
		cur.read_to_end(&mut remaining_data)?;

		Ok(Self {
			version,
			file_name,
			remaining_data
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		PascalString::new("cImageData").write::<u8>(writer)?;
		u32::from(TypeId::Txtr).write_le(writer)?;
		self.version.write_le(writer)?;

		(SGResource { file_name: self.file_name.clone() }).write(writer)?;

		self.remaining_data.write(writer)?;

		Ok(())
	}
}
