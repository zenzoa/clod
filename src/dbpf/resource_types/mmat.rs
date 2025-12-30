use std::error::Error;
use std::io::Cursor;

use crate::dbpf::{ Identifier, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::cpf::{ Cpf, CpfType, PropertyValue };

#[derive(Clone)]
pub struct Mmat {
	pub id: Identifier,
	pub flags: u32,
	pub name: PascalString,
	pub copyright: Option<PascalString>,
	pub creator: PascalString,
	pub material_type: PascalString,
	pub object_guid: u32,
	pub model_name: PascalString,
	pub material_state_flags: u32,
	pub object_state_index: i32,
	pub family: PascalString,
	pub subset_name: PascalString,
	pub default_material: bool
}

impl Mmat {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let cpf = Cpf::read(&resource.data)?;

		let flags = match cpf.get_prop("flags") {
			Some(PropertyValue::Uint(val)) => val.clone(),
			_ => return Err("MMAT is missing \"flags\" property.".into())
		};

		let name = match cpf.get_prop("name") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("MMAT is missing \"name\" property.".into())
		};

		let copyright = match cpf.get_prop("copyright") {
			Some(PropertyValue::String(val)) => Some(val.clone()),
			_ => None
		};

		let creator = match cpf.get_prop("creator") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("MMAT is missing \"creator\" property.".into())
		};

		let material_type = match cpf.get_prop("type") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("MMAT is missing \"type\" property.".into())
		};

		let object_guid = match cpf.get_prop("objectGUID") {
			Some(PropertyValue::Uint(val)) => val.clone(),
			_ => return Err("MMAT is missing \"objectGUID\" property.".into())
		};

		let model_name = match cpf.get_prop("modelName") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("MMAT is missing \"modelName\" property.".into())
		};

		let material_state_flags = match cpf.get_prop("materialStateFlags") {
			Some(PropertyValue::Uint(val)) => val.clone(),
			_ => return Err("MMAT is missing \"materialStateFlags\" property.".into())
		};

		let object_state_index = match cpf.get_prop("objectStateIndex") {
			Some(PropertyValue::Int(val)) => val.clone(),
			_ => return Err("MMAT is missing \"objectStateIndex\" property.".into())
		};

		let family = match cpf.get_prop("family") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("MMAT is missing \"family\" property.".into())
		};

		let subset_name = match cpf.get_prop("subsetName") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("MMAT is missing \"subsetName\" property.".into())
		};

		let default_material = match cpf.get_prop("defaultMaterial") {
			Some(PropertyValue::Bool(val)) => val.clone(),
			_ => return Err("MMAT is missing \"defaultMaterial\" property.".into())
		};

		Ok(Self {
			id: resource.id.clone(),
			flags,
			name,
			copyright,
			creator,
			material_type,
			object_guid,
			model_name,
			material_state_flags,
			object_state_index,
			family,
			subset_name,
			default_material
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let mut cur = Cursor::new(Vec::new());

		let mut props = Vec::new();
		props.push(("flags".to_string(), PropertyValue::Uint(self.flags)));
		props.push(("name".to_string(), PropertyValue::String(self.name.clone())));
		if let Some(copyright) = &self.copyright {
			props.push(("copyright".to_string(), PropertyValue::String(copyright.clone())));
		}
		props.push(("creator".to_string(), PropertyValue::String(self.creator.clone())));
		props.push(("type".to_string(), PropertyValue::String(self.material_type.clone())));
		props.push(("objectGUID".to_string(), PropertyValue::Uint(self.object_guid)));
		props.push(("modelName".to_string(), PropertyValue::String(self.model_name.clone())));
		props.push(("materialStateFlags".to_string(), PropertyValue::Uint(self.material_state_flags)));
		props.push(("objectStateIndex".to_string(), PropertyValue::Int(self.object_state_index)));
		props.push(("family".to_string(), PropertyValue::String(self.family.clone())));
		props.push(("subsetName".to_string(), PropertyValue::String(self.subset_name.clone())));
		props.push(("defaultMaterial".to_string(), PropertyValue::Bool(self.default_material)));

		let cpf = Cpf {
			cpf_type: CpfType::Normal,
			version: Some(2),
			props
		};

		cpf.write(&mut cur)?;

		Ok(cur.into_inner())
	}
}
