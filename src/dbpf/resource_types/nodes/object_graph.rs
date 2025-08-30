use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::SevenBitString;

#[derive(Clone)]
struct ObjectGraphNodeExtension {
	pub is_enabled: bool,
	pub is_dependent: bool,
	pub extension_index: u32,
}

#[derive(Clone)]
pub struct ObjectGraphNode {
	extensions: Vec<ObjectGraphNodeExtension>,
	file_name: Option<SevenBitString>
}

impl ObjectGraphNode {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let block_name = SevenBitString::read(cur)?;
		if &block_name.to_string() != "cObjectGraphNode" {
			return Err(format!("Invalid cObjectGraphNode header.").into());
		}

		let _block_id = u32::read_le(cur)?; // expect 0
		let version = u32::read_le(cur)?; // expect 3 or 4

		let num_extensions = u32::read_le(cur)?;
		let mut extensions = Vec::new();
		for _ in 0..num_extensions {
			let is_enabled = u8::read(cur)? != 0;
			let is_dependent = u8::read(cur)? != 0;
			let extension_index = u32::read_le(cur)?;
			extensions.push(ObjectGraphNodeExtension {
				is_enabled,
				is_dependent,
				extension_index
			})
		}

		let file_name = if version == 4 {
			Some(SevenBitString::read(cur)?)
		} else {
			None
		};

		Ok(Self {
			extensions,
			file_name
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		SevenBitString::new("cObjectGraphNode").write(writer)?;
		0u32.write_le(writer)?;

		let version = if self.file_name.is_none() { 3 } else { 4 };
		version.write_le(writer)?;

		(self.extensions.len() as u32).write_le(writer)?;
		for extension in &self.extensions {
			(if extension.is_enabled { 1u8 } else { 0u8 }).write(writer)?;
			(if extension.is_dependent { 1u8 } else { 0u8 }).write(writer)?;
			extension.extension_index.write_le(writer)?;
		}

		if let Some(file_name) = &self.file_name {
			file_name.write(writer)?;
		}

		Ok(())
	}
}
