use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ TypeId, PascalString };
use crate::dbpf::resource_types::nodes::transform::TransformNode;

#[derive(Clone)]
pub struct ShapeRefNode {
	version: u32,
	transform_node: TransformNode,
	links: Vec<(bool, bool, u32)>,
	unknown: u32,
	blend_names: Vec<PascalString>,
	padding_size: u32
}

impl ShapeRefNode {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		println!("ShapeRefNode");
		let block_name = PascalString::read::<u8>(cur)?;
		if &block_name.to_string() != "cShapeRefNode" {
			return Err(format!("Invalid cShapeRefNode header.").into());
		}
		let _block_id = u32::read_le(cur)?; // expect 0x65245517
		let version = u32::read_le(cur)?; // expect 20 or 21
		println!("  version: {}", version);

		let _block_name2 = PascalString::read::<u8>(cur)?; // expect "cRenderableNode"
		let _block_id2 = u32::read_le(cur)?; // expect 0
		let _version2 = u32::read_le(cur)?; // expect 5

		let _block_name3 = PascalString::read::<u8>(cur)?; // expect "cBoundedNode"
		let _block_id3 = u32::read_le(cur)?; // expect 0
		let _version3 = u32::read_le(cur)?; // expect 5

		let transform_node = TransformNode::read(cur)?;

		let _unknown = u16::read_le(cur)?; // expect 1
		let _unknown = u32::read_le(cur)?; // expect 1
		let _practical = PascalString::read::<u8>(cur)?; // expect "Practical"
		let _unknown = u32::read_le(cur)?; // expect 0
		let _unknown = u8::read(cur)?; // expect 1

		let num_links = u32::read_le(cur)?;
		println!("  num_links: {}", num_links);
		let mut links = Vec::new();
		for _ in 0..num_links {
			let is_enabled = u8::read(cur)? == 1;
			let is_dependent = u8::read(cur)? == 1;
			let link_index = u32::read_le(cur)?;
			links.push((is_enabled, is_dependent, link_index));
		}

		let unknown = u32::read_le(cur)?; // expect 0 or 16
		println!("  unknown: {}", unknown);

		let num_blend_names = u32::read_le(cur)?;
		println!("  num_blend_names: {}", num_blend_names);
		let mut blend_names = Vec::new();
		for _ in 0..num_blend_names { u32::read_le(cur)?; }
		for _ in 0..num_blend_names {
			if version == 21 {
				blend_names.push(PascalString::read::<u8>(cur)?);
				println!("  blend_name: {}", blend_names.last().unwrap());
			} else {
				blend_names.push(PascalString::new(""));
			}
		}

		let padding_size = u32::read_le(cur)?;
		println!("  padding_size: {}", padding_size);
		for _ in 0..padding_size { u8::read(cur)?; }

		println!("  Ok!");

		Ok(Self {
			version,
			transform_node,
			links,
			unknown,
			blend_names,
			padding_size
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		PascalString::new("cDataListExtension").write::<u8>(writer)?;
		(TypeId::ShapeRef as u32).write_le(writer)?;
		self.version.write_le(writer)?;

		PascalString::new("cRenderableNode").write::<u8>(writer)?;
		0u32.write_le(writer)?;
		5u32.write_le(writer)?;

		PascalString::new("cBoundedNode").write::<u8>(writer)?;
		0u32.write_le(writer)?;
		5u32.write_le(writer)?;

		self.transform_node.write(writer)?;

		1u16.write_le(writer)?;
		1u32.write_le(writer)?;
		PascalString::new("Practical").write::<u8>(writer)?;
		0u32.write_le(writer)?;
		1u8.write(writer)?;

		(self.links.len() as u32).write_le(writer)?;
		for (is_enabled, is_dependent, link_index) in &self.links {
			if *is_enabled { 1u8.write(writer)?; } else { 0u8.write(writer)?; }
			if *is_dependent { 1u8.write(writer)?; } else { 0u8.write(writer)?; }
			link_index.write_le(writer)?;
		}

		self.unknown.write_le(writer)?;

		(self.blend_names.len() as u32).write_le(writer)?;
		for _ in &self.blend_names { 0u32.write_le(writer)?; }
		if self.version == 21 {
			for blend_name in &self.blend_names {
				blend_name.write::<u8>(writer)?;
			}
		}

		self.padding_size.write_le(writer)?;
		for _ in 0..self.padding_size { 1u8.write(writer)?; }

		0xffffffffu32.write_le(writer)?;

		Ok(())
	}
}
