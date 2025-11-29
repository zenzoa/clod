use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ Identifier, TypeId };

use crate::dbpf::resource_types::gmdc::GmdcBlock;
use crate::dbpf::resource_types::gmnd::GmndBlock;
use crate::dbpf::resource_types::shpe::ShpeBlock;
use crate::dbpf::resource_types::cres::CresBlock;
use crate::dbpf::resource_types::txmt::TxmtBlock;
use crate::dbpf::resource_types::txtr::TxtrBlock;

pub struct Rcol {
	pub links: Vec<Identifier>,
	pub blocks: Vec<RcolBlock>
}

impl Rcol {
	pub fn read(data: &[u8]) -> Result<Self, Box<dyn Error>> {
		let mut cur = Cursor::new(data);
		let first = u32::read_le(&mut cur)?;
		let use_tgir = first == 0xFFFF0001;
		let num_links = if use_tgir { u32::read_le(&mut cur)? } else { first };
		let mut links = Vec::new();
		for _ in 0..num_links {
			let group_id = u32::read_le(&mut cur)?;
			let instance_id = u32::read_le(&mut cur)?;
			let resource_id = if use_tgir { u32::read_le(&mut cur)? } else { 0 };
			let type_id = u32::read_le(&mut cur)?;
			links.push(Identifier::new(type_id, group_id, instance_id, resource_id));
		}

		let num_blocks = u32::read_le(&mut cur)?;
		let mut block_ids = Vec::new();
		for _ in 0..num_blocks {
			let block_id = u32::read_le(&mut cur)?;
			block_ids.push(block_id);
		}

		let mut blocks = Vec::new();
		for block_id in block_ids {
			blocks.push(RcolBlock::read(&mut cur, block_id)?);
		}

		Ok(Self {
			links,
			blocks
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		0xFFFF0001u32.write_le(writer)?;

		(self.links.len() as u32).write_le(writer)?;
		for link in &self.links {
			link.group_id.write_le(writer)?;
			link.instance_id.write_le(writer)?;
			link.resource_id.write_le(writer)?;
			u32::from(link.type_id).write_le(writer)?;
		}

		(self.blocks.len() as u32).write_le(writer)?;
		for block in &self.blocks {
			block.write_id(writer)?;
		}

		for block in &self.blocks {
			block.write(writer)?;
		}

		Ok(())
	}
}

#[derive(Clone)]
pub enum RcolBlock {
	Gmdc(GmdcBlock),
	Gmnd(GmndBlock),
	Shpe(ShpeBlock),
	Cres(CresBlock),
	Txmt(TxmtBlock),
	Txtr(TxtrBlock),
	Unknown(())
}

impl RcolBlock {
	pub fn read(cur: &mut Cursor<&[u8]>, block_id: u32) -> Result<RcolBlock, Box<dyn Error>> {
		match TypeId::from(block_id) {
			TypeId::Gmdc => {
				let gmdc_block = GmdcBlock::read(cur)?;
				Ok(RcolBlock::Gmdc(gmdc_block))
			},
			TypeId::Gmnd => {
				let gmnd_block = GmndBlock::read(cur)?;
				Ok(RcolBlock::Gmnd(gmnd_block))
			},
			TypeId::Shpe => {
				let shpe_block = ShpeBlock::read(cur)?;
				Ok(RcolBlock::Shpe(shpe_block))
			},
			TypeId::Cres => {
				let cres_block = CresBlock::read(cur)?;
				Ok(RcolBlock::Cres(cres_block))
			},
			TypeId::Txmt => {
				let txmt_block = TxmtBlock::read(cur)?;
				Ok(RcolBlock::Txmt(txmt_block))
			},
			TypeId::Txtr => {
				let txtr_block = TxtrBlock::read(cur)?;
				Ok(RcolBlock::Txtr(txtr_block))
			},
			_ => {
				Ok(RcolBlock::Unknown(()))
			}
		}
	}

	pub fn write_id(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		match self {
			RcolBlock::Gmdc(_) => u32::from(TypeId::Gmdc).write_le(writer)?,
			RcolBlock::Gmnd(_) => u32::from(TypeId::Gmnd).write_le(writer)?,
			RcolBlock::Shpe(_) => u32::from(TypeId::Shpe).write_le(writer)?,
			RcolBlock::Cres(_) => u32::from(TypeId::Cres).write_le(writer)?,
			RcolBlock::Txmt(_) => u32::from(TypeId::Txmt).write_le(writer)?,
			RcolBlock::Txtr(_) => u32::from(TypeId::Txtr).write_le(writer)?,
			RcolBlock::Unknown(block_id) => block_id.write_le(writer)?
		}
		Ok(())
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		match self {
			RcolBlock::Gmdc(gmdc_block) => gmdc_block.write(writer)?,
			RcolBlock::Gmnd(gmnd_block) => gmnd_block.write(writer)?,
			RcolBlock::Shpe(shpe_block) => shpe_block.write(writer)?,
			RcolBlock::Cres(cres_block) => cres_block.write(writer)?,
			RcolBlock::Txmt(txmt_block) => txmt_block.write(writer)?,
			RcolBlock::Txtr(txtr_block) => txtr_block.write(writer)?,
			RcolBlock::Unknown(_block_id) => {}
		}
		Ok(())
	}
}
