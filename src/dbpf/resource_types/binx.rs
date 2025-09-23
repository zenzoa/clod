use std::error::Error;
use std::io::Cursor;
use std::collections::HashMap;

use crate::dbpf::{ Identifier, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::cpf::{ Cpf, CpfType, PropertyValue };

#[derive(Clone)]
pub struct Binx {
	pub id: Identifier,
	pub icon_idx: u32,
	pub stringset_idx: u32,
	pub bin_idx: u32,
	pub object_idx: u32,
	pub creator_id: PascalString,
	pub sort_index: u32,
	pub string_index: u32
}

impl Binx {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let cpf = Cpf::read(&resource.data)?;

		let icon_idx = if let Some(PropertyValue::Uint(v)) = cpf.props.get("iconidx") {
			*v
		} else {
			return Err("BINX has no iconidx value".into());
		};

		let stringset_idx = if let Some(PropertyValue::Uint(v)) = cpf.props.get("stringsetidx") {
			*v
		} else {
			return Err("BINX has no stringsetidx value".into());
		};

		let bin_idx = if let Some(PropertyValue::Uint(v)) = cpf.props.get("binidx") {
			*v
		} else {
			return Err("BINX has no binidx value".into());
		};

		let object_idx = if let Some(PropertyValue::Uint(v)) = cpf.props.get("objectidx") {
			*v
		} else {
			return Err("BINX has no objectidx value".into());
		};

		let creator_id = if let Some(PropertyValue::String(v)) = cpf.props.get("creatorid") {
			v.clone()
		} else {
			PascalString::new("00000000-0000-0000-0000-000000000000")
		};

		let sort_index = if let Some(PropertyValue::Uint(v)) = cpf.props.get("sortindex") {
			*v
		} else {
			0
		};

		let string_index = if let Some(PropertyValue::Uint(v)) = cpf.props.get("stringindex") {
			*v
		} else {
			1
		};

		Ok(Self {
			id: resource.id.clone(),
			icon_idx,
			stringset_idx,
			bin_idx,
			object_idx,
			creator_id,
			sort_index,
			string_index
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let bytes: Vec<u8> = Vec::new();
		let mut cur = Cursor::new(bytes);

		let cpf = Cpf {
			cpf_type: CpfType::Normal,
			version: Some(0),
			props: HashMap::<String, PropertyValue>::from([
				("iconidx".to_string(), PropertyValue::Uint(self.icon_idx)),
				("stringsetidx".to_string(), PropertyValue::Uint(self.stringset_idx)),
				("binidx".to_string(), PropertyValue::Uint(self.bin_idx)),
				("objectidx".to_string(), PropertyValue::Uint(self.object_idx)),
				("creatorid".to_string(), PropertyValue::String(self.creator_id.clone())),
				("sortindex".to_string(), PropertyValue::Uint(self.sort_index)),
				("stringindex".to_string(), PropertyValue::Uint(self.string_index))
			])
		};

		cpf.write(&mut cur)?;

		Ok(cur.into_inner())
	}
}
