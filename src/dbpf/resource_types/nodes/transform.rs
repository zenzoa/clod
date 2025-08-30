use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ TypeId, SevenBitString, Transformation, Quaternion };
use crate::dbpf::resource_types::nodes::object_graph::ObjectGraphNode;

#[derive(Clone)]
pub struct TransformNode {
	pub ogn: ObjectGraphNode,
	pub subnodes: Vec<(bool, bool, u32)>,
	pub transformation: Transformation,
	pub quaternion: Quaternion,
	pub assigned_subset: u32
}

impl TransformNode {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		println!("TransformNode");
		let block_name = SevenBitString::read(cur)?;
		if &block_name.to_string() != "cTransformNode" {
			return Err(format!("Invalid cTransformNode header.").into());
		}
		let block_id = u32::read_le(cur)?; // expect 0x65246462
		println!("  block_id: {:x}", block_id);
		let block_version = u32::read_le(cur)?; // expect 7
		println!("  block_version: {}", block_version);

		let _block_name2 = SevenBitString::read(cur)?; // expect cCompositionTreeNode
		let block_id2 = u32::read_le(cur)?; // expect 0
		println!("  block_id2: {:x}", block_id2);
		let block_version2 = u32::read_le(cur)?; // expect 11
		println!("  block_version2: {}", block_version2);

		let ogn = ObjectGraphNode::read(cur)?;

		let num_subnodes = u32::read_le(cur)?;
		let mut subnodes = Vec::new();
		for _ in 0..num_subnodes {
			let is_enabled = u8::read(cur)? == 1;
			let is_dependent = u8::read(cur)? == 1;
			let subnode_index = u32::read_le(cur)?;
			subnodes.push((is_enabled, is_dependent, subnode_index));
		}

		let transformation = Transformation::read(cur)?;
		let quaternion = Quaternion::read(cur)?;

		let assigned_subset = u32::read_le(cur)?;

		Ok(Self {
			ogn,
			subnodes,
			transformation,
			quaternion,
			assigned_subset
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		SevenBitString::new("cTransformNode").write(writer)?;
		(TypeId::Transform as u32).write_le(writer)?;
		1u8.write(writer)?;

		SevenBitString::new("cCompositionTreeNode").write(writer)?;
		1u8.write(writer)?;
		1u8.write(writer)?;

		self.ogn.write(writer)?;

		(self.subnodes.len() as u32).write_le(writer)?;
		for (is_enabled, is_dependent, subnode_index) in &self.subnodes {
			if *is_enabled { 1u8.write(writer)?; } else { 0u8.write(writer)?; }
			if *is_dependent { 1u8.write(writer)?; } else { 0u8.write(writer)?; }
			subnode_index.write_le(writer)?;
		}

		self.transformation.write(writer)?;
		self.quaternion.write(writer)?;

		self.assigned_subset.write_le(writer)?;
		Ok(())
	}
}
