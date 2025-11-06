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
	pub ui_ref: Option<Identifier>,
	pub str_ref: Option<Identifier>,
	pub coll_ref: Option<Identifier>,
	pub gzps_ref: Option<Identifier>
}

impl Idr {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let mut cres_ref = None;
		let mut shpe_ref = None;
		let mut txmt_refs = Vec::new();
		let mut ui_ref = None;
		let mut str_ref = None;
		let mut coll_ref = None;
		let mut gzps_ref = None;

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
				TypeId::Ui => { ui_ref = Some(entry_id); }
				TypeId::TextList => { str_ref = Some(entry_id); }
				TypeId::Coll => { coll_ref = Some(entry_id); }
				TypeId::Gzps => { gzps_ref = Some(entry_id); }
				_ => {}
			}
		}

		Ok(Self {
			id: resource.id.clone(),
			cres_ref,
			shpe_ref,
			txmt_refs,
			ui_ref,
			str_ref,
			coll_ref,
			gzps_ref
		})
	}

	pub fn new_empty(id: &Identifier) -> Self {
		let mut id = id.clone();
		id.type_id = TypeId::Idr;
		Self {
			id,
			cres_ref: None,
			shpe_ref: None,
			txmt_refs: Vec::new(),
			ui_ref: None,
			str_ref: None,
			coll_ref: None,
			gzps_ref: None
		}
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let mut cur = Cursor::new(Vec::new());

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

		if let Some(ui_ref) = &self.ui_ref {
			entries.push(ui_ref.clone());
		}

		if let Some(str_ref) = &self.str_ref {
			entries.push(str_ref.clone());
		}

		if let Some(coll_ref) = &self.coll_ref {
			entries.push(coll_ref.clone());
		}

		if let Some(gzps_ref) = &self.gzps_ref {
			entries.push(gzps_ref.clone());
		}

		(entries.len() as u32).write_le(&mut cur)?;
		for entry in entries {
			entry.write(&mut cur, true)?;
		}

		Ok(cur.into_inner())
	}
}
