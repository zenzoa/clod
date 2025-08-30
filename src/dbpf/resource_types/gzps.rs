use std::error::Error;
use std::io::Cursor;
use std::collections::HashMap;

use crate::dbpf::{ Identifier, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::cpf::{ Cpf, PropertyValue };

#[derive(Clone, Default)]
pub struct Gzps {
	pub id: Identifier,
	pub cpf_version: u16,

	pub version: Option<u32>,
	pub product: Option<u32>,

	pub age: Vec<Age>,
	pub gender: Vec<Gender>,
	pub species: u32,
	pub outfit: u32,
	pub parts: u32,
	pub flags: u32,
	pub name: PascalString,
	pub creator: PascalString,
	pub family: PascalString,
	pub genetic: Option<f32>,
	pub priority: Option<u32>,
	pub outfit_type: PascalString,
	pub skintone: PascalString,
	pub hairtone: PascalString,
	pub category: Vec<Category>,
	pub shoe: Shoe,
	pub fitness: u32,

	pub resource: u32,
	pub shape: u32,

	pub overrides: Vec<Override>,

	pub title: String
}

impl Gzps {
	pub fn new(resource: &Resource, title: &str) -> Result<Self, Box<dyn Error>> {
		let cpf = Cpf::read(&resource.data)?;
		let mut gzps = Self {
			id: resource.id.clone(),
			cpf_version: cpf.version,
			..Self::default()
		};

		gzps.version = match cpf.props.get("version") {
			Some(PropertyValue::Int(val)) => Some(*val),
			_ => None
		};

		gzps.product = match cpf.props.get("product") {
			Some(PropertyValue::Int(val)) => Some(*val),
			_ => None
		};

		gzps.age = match cpf.props.get("age") {
			Some(PropertyValue::Int(val)) => Age::from_flag(*val),
			_ => return Err("GZPS is missing \"age\" property.".into())
		};

		gzps.gender = match cpf.props.get("gender") {
			Some(PropertyValue::Int(val)) => Gender::from_flag(*val),
			_ => return Err("GZPS is missing \"gender\" property.".into())
		};

		gzps.species = match cpf.props.get("species") {
			Some(PropertyValue::Int(val)) => *val,
			_ => return Err("GZPS is missing \"species\" property.".into())
		};

		let outfit_prop = match cpf.props.get("outfit") {
			Some(PropertyValue::Int(val)) => Some(*val),
			_ => None
		};

		let parts_prop = match cpf.props.get("parts") {
			Some(PropertyValue::Int(val)) => Some(*val),
			_ => None
		};

		(gzps.outfit , gzps.parts) = match (outfit_prop, parts_prop) {
			(Some(outfit), Some(parts)) => (outfit, parts),
			(Some(outfit), None) => (outfit, outfit),
			(None, Some(parts)) => (parts, parts),
			(None, None) => return Err("GZPS is missing both \"outfit\" and \"parts\" properties.".into())
		};

		gzps.flags = match cpf.props.get("flags") {
			Some(PropertyValue::Int(val)) => *val,
			_ => return Err("GZPS is missing \"flags\" property.".into())
		};

		gzps.name = match cpf.props.get("name") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"name\" property.".into())
		};

		gzps.creator = match cpf.props.get("creator") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"creator\" property.".into())
		};

		gzps.family = match cpf.props.get("family") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"family\" property.".into())
		};

		gzps.genetic = match cpf.props.get("genetic") {
			Some(PropertyValue::Float(val)) => Some(*val),
			_ => None
		};

		gzps.priority = match cpf.props.get("priority") {
			Some(PropertyValue::Int(val)) => Some(*val),
			_ => None
		};

		gzps.outfit_type = match cpf.props.get("type") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"type\" property.".into())
		};

		gzps.skintone = match cpf.props.get("skintone") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"skintone\" property.".into())
		};

		gzps.hairtone = match cpf.props.get("hairtone") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"hairtone\" property.".into())
		};

		gzps.category = match cpf.props.get("category") {
			Some(PropertyValue::Int(val)) => Category::from_flag(*val),
			_ => return Err("GZPS is missing \"category\" property.".into())
		};

		gzps.shoe = match cpf.props.get("shoe") {
			Some(PropertyValue::Int(val)) => Shoe::from_flag(*val),
			_ => return Err("GZPS is missing \"shoe\" property.".into())
		};

		gzps.fitness = match cpf.props.get("fitness") {
			Some(PropertyValue::Int(val)) => *val,
			_ => return Err("GZPS is missing \"fitness\" property.".into())
		};

		gzps.shape = match cpf.props.get(&format!("shapekeyidx")) {
			Some(PropertyValue::Int(val)) => *val,
			_ => return Err(format!("GZPS is missing \"shapekeyidx\" property.").into())
		};

		gzps.resource = match cpf.props.get(&format!("resourcekeyidx")) {
			Some(PropertyValue::Int(val)) => *val,
			_ => return Err(format!("GZPS is missing \"resourcekeyidx\" property.").into())
		};

		let num_overrides = match cpf.props.get("numoverrides") {
			Some(PropertyValue::Int(val)) => *val,
			_ => return Err("GZPS is missing \"numoverrides\" property.".into())
		};

		for i in 0..num_overrides {
			let shape = match cpf.props.get(&format!("override{i}shape")) {
				Some(PropertyValue::Int(val)) => *val,
				_ => return Err(format!("GZPS is missing \"override{i}shape\" property.").into())
			};
			let subset = match cpf.props.get(&format!("override{i}subset")) {
				Some(PropertyValue::String(val)) => val.clone(),
				_ => return Err(format!("GZPS is missing \"override{i}subset\" property.").into())
			};
			let resource = match cpf.props.get(&format!("override{i}resourcekeyidx")) {
				Some(PropertyValue::Int(val)) => *val,
				_ => return Err(format!("GZPS is missing \"override{i}resourcekeyidx\" property.").into())
			};
			gzps.overrides.push(Override {
				shape,
				subset,
				resource
			})
		}

		gzps.title = title.to_string();

		Ok(gzps)
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let bytes: Vec<u8> = Vec::new();
		let mut cur = Cursor::new(bytes);

		let mut props: HashMap<String, PropertyValue> = HashMap::new();

		if let Some(version) = self.version {
			props.insert("version".to_string(), PropertyValue::Int(version));
		}
		if let Some(product) = self.product {
			props.insert("product".to_string(), PropertyValue::Int(product));
		}
		props.insert("age".to_string(), PropertyValue::Int(Age::to_flag(&self.age)));
		props.insert("gender".to_string(), PropertyValue::Int(Gender::to_flag(&self.gender)));
		props.insert("species".to_string(), PropertyValue::Int(self.species));
		props.insert("outfit".to_string(), PropertyValue::Int(self.outfit));
		props.insert("parts".to_string(), PropertyValue::Int(self.parts));
		props.insert("flags".to_string(), PropertyValue::Int(self.flags));
		props.insert("name".to_string(), PropertyValue::String(self.name.clone()));
		props.insert("creator".to_string(), PropertyValue::String(self.creator.clone()));
		props.insert("family".to_string(), PropertyValue::String(self.family.clone()));
		if let Some(genetic) = self.genetic {
			props.insert("genetic".to_string(), PropertyValue::Float(genetic));
		}
		if let Some(priority) = self.priority {
			props.insert("priority".to_string(), PropertyValue::Int(priority));
		}
		props.insert("type".to_string(), PropertyValue::String(self.outfit_type.clone()));
		props.insert("skintone".to_string(), PropertyValue::String(self.skintone.clone()));
		props.insert("hairtone".to_string(), PropertyValue::String(self.hairtone.clone()));
		props.insert("category".to_string(), PropertyValue::Int(Category::to_flag(&self.category)));
		props.insert("shoe".to_string(), PropertyValue::Int(self.shoe as u32));
		props.insert("fitness".to_string(), PropertyValue::Int(self.fitness));
		props.insert("resourcekeyidx".to_string(), PropertyValue::Int(self.resource));
		props.insert("shapekeyidx".to_string(), PropertyValue::Int(self.shape));

		props.insert("numoverrides".to_string(), PropertyValue::Int(self.overrides.len() as u32));
		for (i, outfit_override) in self.overrides.iter().enumerate() {
			props.insert(format!("override{i}shape"), PropertyValue::Int(outfit_override.shape));
			props.insert(format!("override{i}subset"), PropertyValue::String(outfit_override.subset.clone()));
			props.insert(format!("override{i}resourcekeyidx"), PropertyValue::Int(outfit_override.resource));
		}

		let cpf = Cpf {
			version: self.cpf_version,
			props
		};
		cpf.write(&mut cur)?;

		Ok(cur.into_inner())
	}
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Category {
	Everyday = 7,
	Swimwear = 8,
	Sleepwear = 16,
	Formal = 32,
	Underwear = 64,
	Skin = 128,
	Maternity = 256,
	Fitness = 512,
	TryOn = 1024,
	Overlay = 2048,
	Outerwear = 4096
}

impl Category {
	pub fn from_flag(flag: u32) -> Vec<Category> {
		let mut categories = Vec::new();
		if flag & Category::Everyday as u32 > 0 { categories.push(Category::Everyday) }
		if flag & Category::Swimwear as u32 > 0 { categories.push(Category::Swimwear) }
		if flag & Category::Sleepwear as u32 > 0 { categories.push(Category::Sleepwear) }
		if flag & Category::Formal as u32 > 0 { categories.push(Category::Formal) }
		if flag & Category::Underwear as u32 > 0 { categories.push(Category::Underwear) }
		if flag & Category::Skin as u32 > 0 { categories.push(Category::Skin) }
		if flag & Category::Maternity as u32 > 0 { categories.push(Category::Maternity) }
		if flag & Category::Fitness as u32 > 0 { categories.push(Category::Fitness) }
		if flag & Category::TryOn as u32 > 0 { categories.push(Category::TryOn) }
		if flag & Category::Overlay as u32 > 0 { categories.push(Category::Overlay) }
		if flag & Category::Outerwear as u32 > 0 { categories.push(Category::Outerwear) }
		categories
	}

	pub fn to_flag(categories: &[Category]) -> u32 {
		let mut flag = 0;
		for category in categories {
			flag += category.clone() as u32;
		}
		flag
	}
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Age {
	Baby = 32,
	Toddler = 1,
	Child = 2,
	Teen = 4,
	YoungAdult = 64,
	Adult = 8,
	Elder = 16
}

impl Age {
	pub fn from_flag(flag: u32) -> Vec<Age> {
		let mut ages = Vec::new();
		if flag & Age::Baby as u32 > 0 { ages.push(Age::Baby) }
		if flag & Age::Toddler as u32 > 0 { ages.push(Age::Toddler) }
		if flag & Age::Child as u32 > 0 { ages.push(Age::Child) }
		if flag & Age::Teen as u32 > 0 { ages.push(Age::Teen) }
		if flag & Age::YoungAdult as u32 > 0 { ages.push(Age::YoungAdult) }
		if flag & Age::Adult as u32 > 0 { ages.push(Age::Adult) }
		if flag & Age::Elder as u32 > 0 { ages.push(Age::Elder) }
		ages
	}

	pub fn to_flag(ages: &[Age]) -> u32 {
		let mut flag = 0;
		for age in ages {
			flag += *age as u32;
		}
		flag
	}

	pub fn are_compatible(a: &[Age], b: &[Age]) -> bool {
		(a.len() == 1 && b.len() == 1 && a[0] == b[0])
			|| (a.contains(&Age::Adult) && (b.contains(&Age::Adult) || b.contains(&Age::YoungAdult)))
			|| (a.contains(&Age::YoungAdult) && (b.contains(&Age::Adult) || b.contains(&Age::YoungAdult)))
			|| (a.contains(&Age::Elder) && b.contains(&Age::Elder))
	}
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Gender {
	Female = 1,
	Male = 2
	// the two genders x_x
}

impl Gender {
	pub fn from_flag(flag: u32) -> Vec<Gender> {
		let mut genders = Vec::new();
		if flag & Gender::Female as u32 > 0 { genders.push(Gender::Female) }
		if flag & Gender::Male as u32 > 0 { genders.push(Gender::Male) }
		genders
	}

	pub fn to_flag(genders: &[Gender]) -> u32 {
		let mut flag = 0;
		for gender in genders {
			flag += *gender as u32;
		}
		flag
	}

	pub fn are_compatible(genders1: &[Gender], genders2: &[Gender], ages: &[Age]) -> bool {
		ages.contains(&Age::Baby) || ages.contains(&Age::Toddler) || ages.contains(&Age::Child)
			|| (genders1.len() == 1 && genders2.contains(&genders1[0]))
			|| (genders1.len() >= 2 && genders2.len() >= 1)
	}
}

#[repr(u32)]
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum Shoe {
	#[default]
	None = 0,
	Barefoot = 1,
	HeavyBoots = 2,
	Heels = 3,
	NormalShoes = 4,
	Sandals = 5,
	Pajamas = 6,
	Armored = 7
}

impl Shoe {
	pub fn from_flag(flag: u32) -> Self {
		match flag {
			1 => Shoe::Barefoot,
			2 => Shoe::HeavyBoots,
			3 => Shoe::Heels,
			4 => Shoe::NormalShoes,
			5 => Shoe::Sandals,
			6 => Shoe::Pajamas,
			7 => Shoe::Armored,
			_ => Shoe::None
		}
	}
}

#[derive(Clone, Default)]
pub struct Override {
	pub shape: u32,
	pub subset: PascalString,
	pub resource: u32,
}
