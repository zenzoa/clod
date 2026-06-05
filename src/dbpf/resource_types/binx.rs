use std::error::Error;
use std::io::Cursor;

use crate::dbpf::{ Identifier, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::cpf::{ Cpf, PropertyValue };

#[derive(Clone)]
pub struct Binx {
	pub id: Identifier,
	pub ui_index: u32,
	pub text_list_index: u32,
	pub coll_index: u32,
	pub gzps_index: u32,
	pub creator: PascalString,
	pub sort: i32,
	pub string_index: u32
}

impl Binx {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let cpf = Cpf::read(&resource.data)?;

		let ui_index = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("iconidx") {
			*v
		} else if let Some(PropertyValue::Uint(v)) = cpf.get_prop("iconid") {
			*v
		} else {
			return Err("BINX has no iconid or iconidx value".into())
		};

		let text_list_index = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("stringsetidx") {
			*v
		} else if let Some(PropertyValue::Uint(v)) = cpf.get_prop("stringsetid") {
			*v
		} else {
			return Err("BINX has no stringsetid or stringsetidx value".into());
		};

		let coll_index = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("binidx") {
			*v
		} else if let Some(PropertyValue::Uint(v)) = cpf.get_prop("binid") {
			*v
		} else {
			return Err("BINX has no binid or binidx value".into());
		};

		let gzps_index = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("objectidx") {
			*v
		} else if let Some(PropertyValue::Uint(v)) = cpf.get_prop("objectid") {
			*v
		} else {
			return Err("BINX has no objectid or objectidx value".into());
		};

		let creator = if let Some(PropertyValue::String(v)) = cpf.get_prop("creatorid") {
			v.clone()
		} else {
			PascalString::new("00000000-0000-0000-0000-000000000000")
		};

		let sort = if let Some(PropertyValue::Int(v)) = cpf.get_prop("sortindex") {
			*v
		} else {
			0
		};

		let string_index = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("stringindex") {
			*v
		} else {
			1
		};

		Ok(Self {
			id: resource.id.clone(),
			ui_index,
			text_list_index,
			coll_index,
			gzps_index,
			creator,
			sort,
			string_index
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let mut cur = Cursor::new(Vec::new());

		let props = vec![
			("iconidx".to_string(), PropertyValue::Uint(self.ui_index)),
			("stringsetidx".to_string(), PropertyValue::Uint(self.text_list_index)),
			("binidx".to_string(), PropertyValue::Uint(self.coll_index)),
			("objectidx".to_string(), PropertyValue::Uint(self.gzps_index)),
			("creatorid".to_string(), PropertyValue::String(self.creator.clone())),
			("sortindex".to_string(), PropertyValue::Int(self.sort)),
			("stringindex".to_string(), PropertyValue::Uint(self.string_index))
		];

		Cpf::write_props(&props, &mut cur)?;

		Ok(cur.into_inner())
	}
}
