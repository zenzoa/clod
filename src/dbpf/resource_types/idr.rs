use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ Identifier, TypeId };
use crate::dbpf::resource::Resource;

#[derive(Clone)]
pub struct Idr {
	pub id: Identifier,
	pub references: Vec<Identifier>
}

impl Idr {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let mut cur = Cursor::new(&resource.data[..]);

		let _ = u32::read_le(&mut cur)?; // expect 0xDEADBEEF

		let use_tgir = u32::read_le(&mut cur)? == 2;

		let mut references = Vec::new();
		let num_entries = u32::read_le(&mut cur)?;
		for _ in 0..num_entries {
			let reference = Identifier::read(&mut cur, use_tgir)?;
			references.push(reference);
		}

		Ok(Self {
			id: resource.id.clone(),
			references
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let mut cur = Cursor::new(Vec::new());

		0xDEADBEEFu32.write_le(&mut cur)?;

		2u32.write_le(&mut cur)?;

		(self.references.len() as u32).write_le(&mut cur)?;

		for reference in &self.references {
			reference.write(&mut cur, true)?;
		}

		Ok(cur.into_inner())
	}

	pub fn refs_by_type(&self, type_id: TypeId) -> Vec<Identifier> {
		self.references
			.iter()
			.filter_map(|r| if r.type_id == type_id { Some(r.clone()) } else { None })
			.collect::<Vec<Identifier>>()
	}
}
