use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ TypeId, SevenBitString, Quaternion };

#[derive(Clone)]
pub struct BoneDataExtension {
	version: u32,
	unknown1: u32,
	unknown2: f32,
	unknown3: u32,
	unknown4: f32,
	rotation: Quaternion
}

impl BoneDataExtension {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		println!("Bone Data");
		let block_name = SevenBitString::read(cur)?;
		println!("  block_name: {}", block_name);
		if &block_name.to_string() != "cBoneDataExtension" {
			return Err(format!("Invalid cBoneDataExtension header.").into());
		}
		let _block_id = u32::read_le(cur)?; // expect 0xE9075BC5
		let version = u32::read_le(cur)?; // expect 4 or 5
		println!("  version: {}", version);

		let block_name2 = SevenBitString::read(cur)?; // expect cExtension
		println!("  block_name2: {}", block_name2);
		let block_id2 = u32::read_le(cur)?; // expect 0
		println!("  block_id2: {}", block_id2);
		let version2 = u32::read_le(cur)?; // expect 3
		println!("  version2: {}", version2);

		let unknown1 = u32::read_le(cur)?;
		let unknown2 = f32::read_le(cur)?;
		let unknown3 = u32::read_le(cur)?;
		let unknown4 = f32::read_le(cur)?;

		let rotation = Quaternion::read(cur)?;

		Ok(Self {
			version,
			unknown1,
			unknown2,
			unknown3,
			unknown4,
			rotation
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		SevenBitString::new("cBoneDataExtension").write(writer)?;
		(TypeId::BoneData as u32).write_le(writer)?;
		self.version.write_le(writer)?;

		SevenBitString::new("cExtension").write(writer)?;
		0u32.write_le(writer)?;
		3u32.write_le(writer)?;

		self.unknown1.write_le(writer)?;
		self.unknown2.write_le(writer)?;
		self.unknown3.write_le(writer)?;
		self.unknown4.write_le(writer)?;

		self.rotation.write(writer)?;

		Ok(())
	}
}
