use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ Identifier, TypeId };
use crate::dbpf::resource::Resource;

#[derive(Clone)]
pub struct Idr {
	pub id: Identifier,
	pub cres_ref: Option<Identifier>,
	pub shpe_ref: Option<Identifier>,
	pub txmt_refs: Vec<Identifier>,
}

impl Idr {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let mut cres_ref = None;
		let mut shpe_ref = None;
		let mut txmt_refs = Vec::new();

		let mut cur = Cursor::new(&resource.data[..]);

		let _ = u32::read_le(&mut cur)?; // expect 0xDEADBEEF

		let use_tgir = u32::read_le(&mut cur)? == 2;

		let num_entries = u32::read_le(&mut cur)?;
		for _ in 0..num_entries {
			let entry_id = Identifier::read(&mut cur, use_tgir)?;
			match entry_id.type_id {
				TypeId::Cres => { cres_ref = Some(entry_id); }
				TypeId::Shpe => { shpe_ref = Some(entry_id); }
				TypeId::Txmt => { txmt_refs.push(entry_id); }
				_ => {}
			}
		}

		Ok(Self {
			id: resource.id.clone(),
			cres_ref,
			shpe_ref,
			txmt_refs
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let bytes: Vec<u8> = Vec::new();
		let mut cur = Cursor::new(bytes);

		0xDEADBEEFu32.write_le(&mut cur)?;

		2u32.write_le(&mut cur)?;

		let mut entries = Vec::new();
		if let Some(cres_ref) = &self.cres_ref {
			entries.push(cres_ref.clone());
		}
		if let Some(shpe_ref) = &self.shpe_ref {
			entries.push(shpe_ref.clone());
		}
		entries.extend_from_slice(&self.txmt_refs);

		(entries.len() as u32).write_le(&mut cur)?;
		println!("\n3IDR");
		for entry in entries {
			println!("  entry: {}", entry);
			entry.write(&mut cur, true)?;
		}

		Ok(cur.into_inner())
	}
}
