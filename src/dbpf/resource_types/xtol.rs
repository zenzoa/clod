use std::error::Error;
use std::io::Cursor;

use crate::dbpf::{ Identifier, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::cpf::{ Cpf, CpfType, PropertyValue };
use crate::dbpf::resource_types::gzps::{ Age, Gender, Category, HairTone };

#[derive(Clone, Default)]
pub struct Xtol {
	pub id: Identifier,
	pub cpf_type: CpfType,
	pub cpf_version: Option<u16>,

	pub version: Option<u32>,
	pub product: Option<u32>,

	pub xtol_type: PascalString,
	pub subtype: u32,

	pub name: PascalString,
	pub creator: PascalString,
	pub family: PascalString,

	pub age: Vec<Age>,
	pub gender: Vec<Gender>,
	pub species: u32,
	pub category: Vec<Category>,
	pub skintone: PascalString,
	pub hairtone: HairTone,
	pub genetic: f32,
	pub flags: u32,
	pub bin: u32,
	pub layer: u32,

	pub materialkey: Option<u32>,
	pub material: Option<u32>,
	pub materialgroup: Option<u32>,
	pub materialrestype: Option<u32>,
}

impl Xtol {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let cpf = Cpf::read(&resource.data)?;
		let mut xtol = Self {
			id: resource.id.clone(),
			cpf_type: cpf.cpf_type,
			cpf_version: cpf.version,
			..Self::default()
		};

		xtol.version = match cpf.get_prop("version") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		xtol.product = match cpf.get_prop("product") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		xtol.xtol_type = match cpf.get_prop("type") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("XTOL is missing \"type\" property.".into())
		};

		xtol.species = match cpf.get_prop("subtype") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("XTOL is missing \"subtype\" property.".into())
		};

		xtol.name = match cpf.get_prop("name") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("XTOL is missing \"name\" property.".into())
		};

		xtol.creator = match cpf.get_prop("creator") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("XTOL is missing \"creator\" property.".into())
		};

		xtol.family = match cpf.get_prop("family") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("XTOL is missing \"family\" property.".into())
		};

		xtol.age = match cpf.get_prop("age") {
			Some(PropertyValue::Uint(val)) => Age::from_flag(*val),
			_ => return Err("XTOL is missing \"age\" property.".into())
		};

		xtol.gender = match cpf.get_prop("gender") {
			Some(PropertyValue::Uint(val)) => Gender::from_flag(*val),
			_ => return Err("XTOL is missing \"gender\" property.".into())
		};

		xtol.species = match cpf.get_prop("species") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("XTOL is missing \"species\" property.".into())
		};

		xtol.category = match cpf.get_prop("category") {
			Some(PropertyValue::Uint(val)) => Category::from_flag(*val),
			_ => return Err("XTOL is missing \"category\" property.".into())
		};

		xtol.skintone = match cpf.get_prop("skintone") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("XTOL is missing \"skintone\" property.".into())
		};

		xtol.hairtone = match cpf.get_prop("hairtone") {
			Some(PropertyValue::String(val)) => HairTone::from_pascal_string(val),
			_ => return Err("XTOL is missing \"hairtone\" property.".into())
		};

		xtol.genetic = match cpf.get_prop("genetic") {
			Some(PropertyValue::Float(val)) => *val,
			_ => return Err("XTOL is missing \"genetic\" property.".into())
		};

		xtol.flags = match cpf.get_prop("flags") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("XTOL is missing \"flags\" property.".into())
		};

		xtol.bin = match cpf.get_prop("bin") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("XTOL is missing \"bin\" property.".into())
		};

		xtol.layer = match cpf.get_prop("layer") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("XTOL is missing \"layer\" property.".into())
		};

		xtol.materialkey = match cpf.get_prop("materialkeyidx") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		xtol.material = match cpf.get_prop("materialid") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		xtol.materialgroup = match cpf.get_prop("materialgroupid") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		xtol.materialrestype = match cpf.get_prop("materialrestypeid") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		Ok(xtol)
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let mut cur = Cursor::new(Vec::new());

		let mut props = Vec::new();

		if let Some(version) = self.version {
			props.push(("version".to_string(), PropertyValue::Uint(version)));
		}
		if let Some(product) = self.product {
			props.push(("product".to_string(), PropertyValue::Uint(product)));
		}

		props.push(("type".to_string(), PropertyValue::String(self.xtol_type.clone())));
		props.push(("subtype".to_string(), PropertyValue::Uint(self.subtype)));

		props.push(("name".to_string(), PropertyValue::String(self.name.clone())));
		props.push(("creator".to_string(), PropertyValue::String(self.creator.clone())));
		props.push(("family".to_string(), PropertyValue::String(self.family.clone())));

		props.push(("age".to_string(), PropertyValue::Uint(Age::to_flag(&self.age))));
		props.push(("gender".to_string(), PropertyValue::Uint(Gender::to_flag(&self.gender))));
		props.push(("species".to_string(), PropertyValue::Uint(self.species)));
		props.push(("category".to_string(), PropertyValue::Uint(Category::to_flag(&self.category))));
		props.push(("skintone".to_string(), PropertyValue::String(self.skintone.clone())));
		props.push(("hairtone".to_string(), PropertyValue::String(self.hairtone.to_pascal_string())));
		props.push(("genetic".to_string(), PropertyValue::Float(self.genetic)));
		props.push(("flags".to_string(), PropertyValue::Uint(self.flags)));
		props.push(("bin".to_string(), PropertyValue::Uint(self.bin)));
		props.push(("layer".to_string(), PropertyValue::Uint(self.layer)));

		if let Some(materialkey) = self.materialkey {
			props.push(("materialkeyidx".to_string(), PropertyValue::Uint(materialkey)));
		}

		if let Some(material) = self.material {
			props.push(("materialid".to_string(), PropertyValue::Uint(material)));
		}

		if let Some(materialgroup) = self.materialgroup {
			props.push(("materialgroupid".to_string(), PropertyValue::Uint(materialgroup)));
		}

		if let Some(materialrestype) = self.materialrestype {
			props.push(("materialrestypeid".to_string(), PropertyValue::Uint(materialrestype)));
		}

		let cpf = Cpf {
			cpf_type: self.cpf_type,
			version: self.cpf_version,
			props
		};
		cpf.write(&mut cur)?;

		Ok(cur.into_inner())
	}
}
