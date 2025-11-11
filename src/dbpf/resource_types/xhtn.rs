use std::error::Error;
use std::io::Cursor;

use crate::dbpf::{ Identifier, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::cpf::{ Cpf, CpfType, PropertyValue };

#[derive(Clone)]
pub struct Xhtn {
	pub id: Identifier,
	pub version: u32,
	pub product: u32,
	pub age: u32,
	pub gender: u32,
	pub species: u32,
	pub parts: u32,
	pub outfit: u32,
	pub flags: u32,
	pub name: PascalString,
	pub creator: PascalString,
	pub family: PascalString,
	pub genetic: f32,
	pub priority: i32,
	pub xhtn_type: PascalString,
	pub preview: PascalString,
	pub proxy: PascalString,
}

impl Xhtn {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let cpf = Cpf::read(&resource.data)?;

		let version = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("version") {
			*v
		} else {
			6
		};

		let product = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("product") {
			*v
		} else {
			0
		};

		let age = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("age") {
			*v
		} else {
			0
		};

		let gender = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("gender") {
			*v
		} else {
			0
		};

		let species = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("species") {
			*v
		} else {
			1
		};

		let parts = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("parts") {
			*v
		} else {
			0
		};

		let outfit = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("outfit") {
			*v
		} else {
			0
		};

		let flags = if let Some(PropertyValue::Uint(v)) = cpf.get_prop("flags") {
			*v
		} else {
			0
		};

		let name = if let Some(PropertyValue::String(v)) = cpf.get_prop("name") {
			v.clone()
		} else {
			PascalString::new("Black")
		};

		let creator = if let Some(PropertyValue::String(v)) = cpf.get_prop("creator") {
			v.clone()
		} else {
			PascalString::new("00000000-0000-0000-0000-000000000000")
		};

		let family = if let Some(PropertyValue::String(v)) = cpf.get_prop("family") {
			v.clone()
		} else {
			PascalString::new("00000000-0000-0000-0000-000000000000")
		};

		let genetic = if let Some(PropertyValue::Float(v)) = cpf.get_prop("genetic") {
			*v
		} else {
			1.0
		};

		let priority = if let Some(PropertyValue::Int(v)) = cpf.get_prop("priority") {
			*v
		} else {
			0
		};

		let xhtn_type = if let Some(PropertyValue::String(v)) = cpf.get_prop("type") {
			v.clone()
		} else {
			PascalString::new("hairtone")
		};

		let preview = if let Some(PropertyValue::String(v)) = cpf.get_prop("preview") {
			v.clone()
		} else {
			PascalString::new("")
		};

		let proxy = if let Some(PropertyValue::String(v)) = cpf.get_prop("proxy") {
			v.clone()
		} else {
			PascalString::new("00000001-0000-0000-0000-000000000000")
		};

		Ok(Self {
			id: resource.id.clone(),
			version,
			product,
			age,
			gender,
			species,
			parts,
			outfit,
			flags,
			name,
			creator,
			family,
			genetic,
			priority,
			xhtn_type,
			preview,
			proxy,
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let mut cur = Cursor::new(Vec::new());

		let cpf = Cpf {
			cpf_type: CpfType::Normal,
			version: Some(0),
			props: vec![
				("version".to_string(), PropertyValue::Uint(self.version)),
				("product".to_string(), PropertyValue::Uint(self.product)),
				("age".to_string(), PropertyValue::Uint(self.age)),
				("gender".to_string(), PropertyValue::Uint(self.gender)),
				("species".to_string(), PropertyValue::Uint(self.species)),
				("parts".to_string(), PropertyValue::Uint(self.parts)),
				("outfit".to_string(), PropertyValue::Uint(self.outfit)),
				("flags".to_string(), PropertyValue::Uint(self.flags)),
				("name".to_string(), PropertyValue::String(self.name.clone())),
				("creator".to_string(), PropertyValue::String(self.creator.clone())),
				("family".to_string(), PropertyValue::String(self.family.clone())),
				("genetic".to_string(), PropertyValue::Float(self.genetic)),
				("priority".to_string(), PropertyValue::Int(self.priority)),
				("type".to_string(), PropertyValue::String(self.xhtn_type.clone())),
				("preview".to_string(), PropertyValue::String(self.preview.clone())),
				("proxy".to_string(), PropertyValue::String(self.proxy.clone())),
			]
		};

		cpf.write(&mut cur)?;

		Ok(cur.into_inner())
	}
}
