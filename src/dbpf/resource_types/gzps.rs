use std::error::Error;
use std::io::Cursor;
use std::collections::HashMap;

use regex::Regex;

use crate::dbpf::{ Identifier, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::cpf::{ Cpf, CpfType, PropertyValue };

#[derive(Clone, Default)]
pub struct Gzps {
	pub id: Identifier,
	pub cpf_type: CpfType,
	pub cpf_version: Option<u16>,

	pub version: Option<u32>,
	pub product: Option<u32>,

	pub age: Vec<Age>,
	pub gender: Vec<Gender>,
	pub species: u32,
	pub outfit: Vec<Part>,
	pub parts: Vec<Part>,
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
			cpf_type: cpf.cpf_type,
			cpf_version: cpf.version,
			..Self::default()
		};

		gzps.version = match cpf.props.get("version") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		gzps.product = match cpf.props.get("product") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		gzps.age = match cpf.props.get("age") {
			Some(PropertyValue::Uint(val)) => Age::from_flag(*val),
			_ => return Err("GZPS is missing \"age\" property.".into())
		};

		gzps.gender = match cpf.props.get("gender") {
			Some(PropertyValue::Uint(val)) => Gender::from_flag(*val),
			_ => return Err("GZPS is missing \"gender\" property.".into())
		};

		gzps.species = match cpf.props.get("species") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"species\" property.".into())
		};

		let outfit_prop = match cpf.props.get("outfit") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		let parts_prop = match cpf.props.get("parts") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		(gzps.outfit , gzps.parts) = match (outfit_prop, parts_prop) {
			(Some(outfit), Some(parts)) => (Part::from_flag(outfit), Part::from_flag(parts)),
			(Some(outfit), None) => (Part::from_flag(outfit), Part::from_flag(outfit)),
			(None, Some(parts)) => (Part::from_flag(parts), Part::from_flag(parts)),
			(None, None) => return Err("GZPS is missing both \"outfit\" and \"parts\" properties.".into())
		};

		gzps.flags = match cpf.props.get("flags") {
			Some(PropertyValue::Uint(val)) => *val,
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
			Some(PropertyValue::Uint(val)) => Some(*val),
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
			Some(PropertyValue::Uint(val)) => Category::from_flag(*val),
			_ => return Err("GZPS is missing \"category\" property.".into())
		};

		gzps.shoe = match cpf.props.get("shoe") {
			Some(PropertyValue::Uint(val)) => Shoe::from_flag(*val),
			_ => return Err("GZPS is missing \"shoe\" property.".into())
		};

		gzps.fitness = match cpf.props.get("fitness") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"fitness\" property.".into())
		};

		gzps.shape = match cpf.props.get("shapekeyidx") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"shapekeyidx\" property.".into())
		};

		gzps.resource = match cpf.props.get("resourcekeyidx") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"resourcekeyidx\" property.".into())
		};

		let num_overrides = match cpf.props.get("numoverrides") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"numoverrides\" property.".into())
		};

		for i in 0..num_overrides {
			let shape = match cpf.props.get(&format!("override{i}shape")) {
				Some(PropertyValue::Uint(val)) => *val,
				_ => return Err(format!("GZPS is missing \"override{i}shape\" property.").into())
			};
			let subset = match cpf.props.get(&format!("override{i}subset")) {
				Some(PropertyValue::String(val)) => val.clone(),
				_ => return Err(format!("GZPS is missing \"override{i}subset\" property.").into())
			};
			let resource = match cpf.props.get(&format!("override{i}resourcekeyidx")) {
				Some(PropertyValue::Uint(val)) => *val,
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
			props.insert("version".to_string(), PropertyValue::Uint(version));
		}
		if let Some(product) = self.product {
			props.insert("product".to_string(), PropertyValue::Uint(product));
		}
		props.insert("age".to_string(), PropertyValue::Uint(Age::to_flag(&self.age)));
		props.insert("gender".to_string(), PropertyValue::Uint(Gender::to_flag(&self.gender)));
		props.insert("species".to_string(), PropertyValue::Uint(self.species));
		props.insert("outfit".to_string(), PropertyValue::Uint(Part::to_flag(&self.outfit)));
		props.insert("parts".to_string(), PropertyValue::Uint(Part::to_flag(&self.parts)));
		props.insert("flags".to_string(), PropertyValue::Uint(self.flags));
		props.insert("name".to_string(), PropertyValue::String(self.name.clone()));
		props.insert("creator".to_string(), PropertyValue::String(self.creator.clone()));
		props.insert("family".to_string(), PropertyValue::String(self.family.clone()));
		if let Some(genetic) = self.genetic {
			props.insert("genetic".to_string(), PropertyValue::Float(genetic));
		}
		if let Some(priority) = self.priority {
			props.insert("priority".to_string(), PropertyValue::Uint(priority));
		}
		props.insert("type".to_string(), PropertyValue::String(self.outfit_type.clone()));
		props.insert("skintone".to_string(), PropertyValue::String(self.skintone.clone()));
		props.insert("hairtone".to_string(), PropertyValue::String(self.hairtone.clone()));
		props.insert("category".to_string(), PropertyValue::Uint(Category::to_flag(&self.category)));
		props.insert("shoe".to_string(), PropertyValue::Uint(self.shoe as u32));
		props.insert("fitness".to_string(), PropertyValue::Uint(self.fitness));
		props.insert("resourcekeyidx".to_string(), PropertyValue::Uint(self.resource));
		props.insert("shapekeyidx".to_string(), PropertyValue::Uint(self.shape));

		props.insert("numoverrides".to_string(), PropertyValue::Uint(self.overrides.len() as u32));
		for (i, outfit_override) in self.overrides.iter().enumerate() {
			props.insert(format!("override{i}shape"), PropertyValue::Uint(outfit_override.shape));
			props.insert(format!("override{i}subset"), PropertyValue::String(outfit_override.subset.clone()));
			props.insert(format!("override{i}resourcekeyidx"), PropertyValue::Uint(outfit_override.resource));
		}

		let cpf = Cpf {
			cpf_type: self.cpf_type,
			version: self.cpf_version,
			props
		};
		cpf.write(&mut cur)?;

		Ok(cur.into_inner())
	}

	pub fn generate_name(&self) -> String {
		let age = Age::stringify(&self.age);
		let gender = Gender::stringify(&self.gender);
		let part = Part::stringify(&self.parts);

		let full_name = self.name.to_string();
		let mut name_without_prefix = full_name.clone();
		let re = Regex::new(r"^(?:CASIE_)?(?:contest_)?[bpctyaeu][mfu](?:body)?(?:bottom)?(?:top)?([a-z,A-Z,0-9]+)_?").unwrap();
		for (_, [inner_name]) in re.captures_iter(&full_name).map(|c| c.extract()) {
			name_without_prefix = inner_name.to_string();
		}

		let mut name_without_ep = name_without_prefix.clone();
		let re2 = Regex::new(r"([a-z,A-Z,0-9]+)ep\d$").unwrap();
		for (_, [inner_name]) in re2.captures_iter(&name_without_prefix).map(|c| c.extract()) {
			name_without_ep = inner_name.to_string();
		}

		format!("{age}{gender}_{part}_{name_without_ep}")
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
	pub fn from_flag(flag: u32) -> Vec<Self> {
		let mut categories = Vec::new();
		if flag & Self::Everyday as u32 > 0 { categories.push(Self::Everyday) }
		if flag & Self::Swimwear as u32 > 0 { categories.push(Self::Swimwear) }
		if flag & Self::Sleepwear as u32 > 0 { categories.push(Self::Sleepwear) }
		if flag & Self::Formal as u32 > 0 { categories.push(Self::Formal) }
		if flag & Self::Underwear as u32 > 0 { categories.push(Self::Underwear) }
		if flag & Self::Skin as u32 > 0 { categories.push(Self::Skin) }
		if flag & Self::Maternity as u32 > 0 { categories.push(Self::Maternity) }
		if flag & Self::Fitness as u32 > 0 { categories.push(Self::Fitness) }
		if flag & Self::TryOn as u32 > 0 { categories.push(Self::TryOn) }
		if flag & Self::Overlay as u32 > 0 { categories.push(Self::Overlay) }
		if flag & Self::Outerwear as u32 > 0 { categories.push(Self::Outerwear) }
		categories
	}

	pub fn to_flag(categories: &[Self]) -> u32 {
		let mut flag = 0;
		for category in categories {
			flag += *category as u32;
		}
		flag
	}

	// pub fn stringify(categories: &[Self]) -> String {
	// 	let mut category_string = String::new();
	// 	if categories.contains(&Self::Everyday) { category_string.push('e'); }
	// 	if categories.contains(&Self::Swimwear) { category_string.push('b'); }
	// 	if categories.contains(&Self::Sleepwear) { category_string.push('s'); }
	// 	if categories.contains(&Self::Formal) { category_string.push('f'); }
	// 	if categories.contains(&Self::Underwear) { category_string.push('u'); }
	// 	if categories.contains(&Self::Maternity) { category_string.push('m'); }
	// 	if categories.contains(&Self::Fitness) { category_string.push('a'); }
	// 	if categories.contains(&Self::Outerwear) { category_string.push('o'); }
	// 	category_string
	// }
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
	pub fn from_flag(flag: u32) -> Vec<Self> {
		let mut ages = Vec::new();
		if flag & Self::Baby as u32 > 0 { ages.push(Self::Baby) }
		if flag & Self::Toddler as u32 > 0 { ages.push(Self::Toddler) }
		if flag & Self::Child as u32 > 0 { ages.push(Self::Child) }
		if flag & Self::Teen as u32 > 0 { ages.push(Self::Teen) }
		if flag & Self::YoungAdult as u32 > 0 { ages.push(Self::YoungAdult) }
		if flag & Self::Adult as u32 > 0 { ages.push(Self::Adult) }
		if flag & Self::Elder as u32 > 0 { ages.push(Self::Elder) }
		ages
	}

	pub fn to_flag(ages: &[Self]) -> u32 {
		let mut flag = 0;
		for age in ages {
			flag += *age as u32;
		}
		flag
	}

	pub fn are_compatible(a: &[Self], b: &[Self]) -> bool {
		(a.len() == 1 && b.len() == 1 && a[0] == b[0]) ||
			(a.contains(&Self::Adult) && (b.contains(&Self::Adult) || b.contains(&Self::YoungAdult))) ||
			(a.contains(&Self::YoungAdult) && (b.contains(&Self::Adult) || b.contains(&Self::YoungAdult))) ||
			(a.contains(&Self::Elder) && b.contains(&Self::Elder))
	}

	pub fn stringify(ages: &[Self]) -> String {
		let mut age_string = String::new();
		if ages.contains(&Self::Baby) { age_string.push('b'); }
		if ages.contains(&Self::Toddler) { age_string.push('p'); }
		if ages.contains(&Self::Child) { age_string.push('c'); }
		if ages.contains(&Self::Teen) { age_string.push('t'); }
		if ages.contains(&Self::Adult) || ages.contains(&Age::YoungAdult) { age_string.push('a'); }
		if ages.contains(&Self::Elder) { age_string.push('e'); }
		age_string
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
	pub fn from_flag(flag: u32) -> Vec<Self> {
		let mut genders = Vec::new();
		if flag & Self::Female as u32 > 0 { genders.push(Self::Female) }
		if flag & Self::Male as u32 > 0 { genders.push(Self::Male) }
		genders
	}

	pub fn to_flag(genders: &[Self]) -> u32 {
		let mut flag = 0;
		for gender in genders {
			flag += *gender as u32;
		}
		flag
	}

	pub fn are_compatible(genders1: &[Self], genders2: &[Self], ages: &[Age]) -> bool {
		ages.contains(&Age::Baby) || ages.contains(&Age::Toddler) || ages.contains(&Age::Child) ||
			(genders1.len() == 1 && genders2.contains(&genders1[0])) ||
			(genders1.len() >= 2 && !genders2.is_empty())
	}

	pub fn stringify(genders: &[Self]) -> String {
		(if genders.len() > 1 { "u" }
		else if genders.contains(&Self::Male) { "m" }
		else if genders.contains(&Self::Female) { "f" }
		else { "" })
			.to_string()
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

#[repr(u32)]
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum Part {
	#[default]
	None = 0,
	Hair = 1,
	Face = 2,
	Top = 4,
	Body = 8,
	Bottom = 16,
	Accessory = 32,
	TailLong = 64,
	EarsUp = 128,
	TailShort = 256,
	EarsDown = 512,
	BrushTailLong = 1024,
	BrushTailShort = 2048,
	SpitzTail = 4096,
	BrushSpitzTail = 8192
}

impl Part {
	pub fn from_flag(flag: u32) -> Vec<Self> {
		let mut parts = Vec::new();
		if flag & Self::Hair as u32 > 0 { parts.push(Self::Hair) }
		if flag & Self::Face as u32 > 0 { parts.push(Self::Face) }
		if flag & Self::Top as u32 > 0 { parts.push(Self::Top) }
		if flag & Self::Body as u32 > 0 { parts.push(Self::Body) }
		if flag & Self::Bottom as u32 > 0 { parts.push(Self::Bottom) }
		if flag & Self::Accessory as u32 > 0 { parts.push(Self::Accessory) }
		if flag & Self::TailLong as u32 > 0 { parts.push(Self::TailLong) }
		if flag & Self::EarsUp as u32 > 0 { parts.push(Self::EarsUp) }
		if flag & Self::TailShort as u32 > 0 { parts.push(Self::TailShort) }
		if flag & Self::EarsDown as u32 > 0 { parts.push(Self::EarsDown) }
		if flag & Self::BrushTailLong as u32 > 0 { parts.push(Self::BrushTailLong) }
		if flag & Self::BrushTailShort as u32 > 0 { parts.push(Self::BrushTailShort) }
		if flag & Self::SpitzTail as u32 > 0 { parts.push(Self::SpitzTail) }
		if flag & Self::BrushSpitzTail as u32 > 0 { parts.push(Self::BrushSpitzTail) }
		parts
	}

	pub fn to_flag(parts: &[Self]) -> u32 {
		let mut flag = 0;
		for part in parts {
			flag += *part as u32;
		}
		flag
	}

	pub fn stringify(parts: &[Self]) -> String {
		(if parts.contains(&Self::Hair) { "hair" }
		else if parts.contains(&Self::Face) { "face" }
		else if parts.contains(&Self::Top) { "top" }
		else if parts.contains(&Self::Body) { "body" }
		else if parts.contains(&Self::Bottom) { "bottom" }
		else if parts.contains(&Self::Accessory) { "accessory" }
		else { "" })
			.to_string()
	}
}

#[derive(Clone, Default)]
pub struct Override {
	pub shape: u32,
	pub subset: PascalString,
	pub resource: u32,
}
