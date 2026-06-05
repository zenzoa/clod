use std::error::Error;
use std::io::Cursor;

use regex::Regex;

use crate::dbpf::{ Identifier, PascalString };
use crate::dbpf::resource::Resource;
use crate::dbpf::resource_types::cpf::{ Cpf, PropertyValue };

#[derive(Clone, Default)]
pub struct Gzps {
	pub id: Identifier,

	pub version: Option<u32>,
	pub product: Option<u32>,

	pub ages: Vec<Age>,
	pub genders: Vec<Gender>,
	pub species: u32,
	pub outfit: Vec<Part>,
	pub parts: Vec<Part>,
	pub flags: Vec<OutfitFlag>,
	pub name: PascalString,
	pub creator: PascalString,
	pub family: PascalString,
	pub genetic: Option<f32>,
	pub priority: Option<u32>,
	pub outfit_type: PascalString,
	pub skintone: PascalString,
	pub hairtone: HairTone,
	pub categories: Vec<Category>,
	pub shoe: Shoe,
	pub fitness: u32,

	pub cres_index: u32,
	pub shpe_index: u32,
	pub subset_indexes: Vec<SubsetRef>
}

impl Gzps {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let cpf = Cpf::read(&resource.data)?;
		let mut gzps = Self {
			id: resource.id.clone(),
			..Self::default()
		};

		gzps.version = match cpf.get_prop("version") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		gzps.product = match cpf.get_prop("product") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		gzps.ages = match cpf.get_prop("age") {
			Some(PropertyValue::Uint(val)) => Age::from_flag(*val),
			_ => return Err("GZPS is missing \"age\" property.".into())
		};

		gzps.genders = match cpf.get_prop("gender") {
			Some(PropertyValue::Uint(val)) => Gender::from_flag(*val),
			_ => return Err("GZPS is missing \"gender\" property.".into())
		};

		gzps.species = match cpf.get_prop("species") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"species\" property.".into())
		};

		let outfit_prop = match cpf.get_prop("outfit") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		let parts_prop = match cpf.get_prop("parts") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		(gzps.outfit , gzps.parts) = match (outfit_prop, parts_prop) {
			(Some(outfit), Some(parts)) => (Part::from_flag(outfit), Part::from_flag(parts)),
			(Some(outfit), None) => (Part::from_flag(outfit), Part::from_flag(outfit)),
			(None, Some(parts)) => (Part::from_flag(parts), Part::from_flag(parts)),
			(None, None) => return Err("GZPS is missing both \"outfit\" and \"parts\" properties.".into())
		};

		gzps.flags = match cpf.get_prop("flags") {
			Some(PropertyValue::Uint(val)) => OutfitFlag::from_flag(*val),
			_ => return Err("GZPS is missing \"flags\" property.".into())
		};

		gzps.name = match cpf.get_prop("name") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"name\" property.".into())
		};

		gzps.creator = match cpf.get_prop("creator") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"creator\" property.".into())
		};

		gzps.family = match cpf.get_prop("family") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"family\" property.".into())
		};

		gzps.genetic = match cpf.get_prop("genetic") {
			Some(PropertyValue::Float(val)) => Some(*val),
			_ => None
		};

		gzps.priority = match cpf.get_prop("priority") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		gzps.outfit_type = match cpf.get_prop("type") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"type\" property.".into())
		};

		gzps.skintone = match cpf.get_prop("skintone") {
			Some(PropertyValue::String(val)) => val.clone(),
			_ => return Err("GZPS is missing \"skintone\" property.".into())
		};

		gzps.hairtone = match cpf.get_prop("hairtone") {
			Some(PropertyValue::String(val)) => HairTone::from_pascal_string(val),
			_ => return Err("GZPS is missing \"hairtone\" property.".into())
		};

		gzps.categories = match cpf.get_prop("category") {
			Some(PropertyValue::Uint(val)) => Category::from_flag(*val),
			_ => return Err("GZPS is missing \"category\" property.".into())
		};

		gzps.shoe = match cpf.get_prop("shoe") {
			Some(PropertyValue::Uint(val)) => Shoe::from_flag(*val),
			_ => return Err("GZPS is missing \"shoe\" property.".into())
		};

		gzps.fitness = match cpf.get_prop("fitness") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"fitness\" property.".into())
		};

		gzps.cres_index = match cpf.get_prop("resourcekeyidx") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"resourcekeyidx\" property.".into())
		};

		gzps.shpe_index = match cpf.get_prop("shapekeyidx") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"shapekeyidx\" property.".into())
		};

		let num_overrides = match cpf.get_prop("numoverrides") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"numoverrides\" property.".into())
		};

		for i in 0..num_overrides {
			let shpe_index = match cpf.get_prop(&format!("override{i}shape")) {
				Some(PropertyValue::Uint(val)) => *val,
				_ => return Err(format!("GZPS is missing \"override{i}shape\" property.").into())
			};
			let subset_name = match cpf.get_prop(&format!("override{i}subset")) {
				Some(PropertyValue::String(val)) => val.clone(),
				_ => return Err(format!("GZPS is missing \"override{i}subset\" property.").into())
			};
			let txmt_index = match cpf.get_prop(&format!("override{i}resourcekeyidx")) {
				Some(PropertyValue::Uint(val)) => *val,
				_ => return Err(format!("GZPS is missing \"override{i}resourcekeyidx\" property.").into())
			};
			gzps.subset_indexes.push(SubsetRef {
				shpe_index,
				subset_name,
				txmt_index
			})
		}

		Ok(gzps)
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
		props.push(("age".to_string(), PropertyValue::Uint(Age::to_flag(&self.ages))));
		props.push(("gender".to_string(), PropertyValue::Uint(Gender::to_flag(&self.genders))));
		props.push(("species".to_string(), PropertyValue::Uint(self.species)));
		props.push(("outfit".to_string(), PropertyValue::Uint(Part::to_flag(&self.outfit))));
		props.push(("parts".to_string(), PropertyValue::Uint(Part::to_flag(&self.parts))));
		props.push(("flags".to_string(), PropertyValue::Uint(OutfitFlag::to_flag(&self.flags))));
		props.push(("name".to_string(), PropertyValue::String(self.name.clone())));
		props.push(("creator".to_string(), PropertyValue::String(self.creator.clone())));
		props.push(("family".to_string(), PropertyValue::String(self.family.clone())));
		if let Some(genetic) = self.genetic {
			props.push(("genetic".to_string(), PropertyValue::Float(genetic)));
		}
		if let Some(priority) = self.priority {
			props.push(("priority".to_string(), PropertyValue::Uint(priority)));
		}
		props.push(("type".to_string(), PropertyValue::String(self.outfit_type.clone())));
		props.push(("skintone".to_string(), PropertyValue::String(self.skintone.clone())));
		props.push(("hairtone".to_string(), PropertyValue::String(self.hairtone.to_pascal_string())));
		props.push(("category".to_string(), PropertyValue::Uint(Category::to_flag(&self.categories))));
		props.push(("shoe".to_string(), PropertyValue::Uint(self.shoe as u32)));
		props.push(("fitness".to_string(), PropertyValue::Uint(self.fitness)));
		props.push(("resourcekeyidx".to_string(), PropertyValue::Uint(self.cres_index)));
		props.push(("shapekeyidx".to_string(), PropertyValue::Uint(self.shpe_index)));

		props.push(("numoverrides".to_string(), PropertyValue::Uint(self.subset_indexes.len() as u32)));
		for (i, outfit_override) in self.subset_indexes.iter().enumerate() {
			props.push((format!("override{i}shape"), PropertyValue::Uint(outfit_override.shpe_index)));
			props.push((format!("override{i}subset"), PropertyValue::String(outfit_override.subset_name.clone())));
			props.push((format!("override{i}resourcekeyidx"), PropertyValue::Uint(outfit_override.txmt_index)));
		}

		Cpf::write_props(&props, &mut cur)?;

		Ok(cur.into_inner())
	}

	pub fn outfit_name(&self) -> String {
		format!("{}_{:08X}-{:08X}-{:08X}", self.name, self.id.group_id, self.id.resource_id, self.id.instance_id)
	}

	pub fn outfit_group_name(&self) -> String {
		let age = Age::stringify(&self.ages);
		let gender = Gender::stringify(&self.genders);
		let part = Part::stringify(&self.parts);

		let full_name = self.name.to_string().to_lowercase().trim().to_string();
		let mut base_name = full_name.clone();

		let re = Regex::new(r"^(?:casie_)?(?:contest_)?[bpctyaeu][mfu](?:body)?(?:bottom)?(?:top)?([a-z,0-9]+)_?").unwrap();
		for (_, [inner]) in re.captures_iter(&full_name).map(|cap| cap.extract()) {
			base_name = inner.to_string();
		}

		format!("{age}{gender}{part}_{base_name}")
	}

	pub fn max_resource_key(&self) -> u32 {
		let resource_keys = self.subset_indexes.iter().map(|o| o.txmt_index);
		resource_keys.max().unwrap_or(0)
	}

	pub fn make_unisex(&mut self) {
		if !self.ages.contains(&Age::Teen) &&
			!self.ages.contains(&Age::YoungAdult) &&
			!self.ages.contains(&Age::Adult) &&
			!self.ages.contains(&Age::Elder) {
				self.genders = vec![Gender::Male, Gender::Female];
		}
	}

	pub fn update_with(&mut self, make_unisex: bool, category_overrides: &Option<Vec<Category>>, flag_overrides: &Option<Vec<OutfitFlag>>, txmt_ids: &[(String, Identifier)]) {
		self.creator = PascalString::new("00000000-0000-0000-0000-000000000000");

		// Set version/product to remove pack icon or custom star and sort with the rest
		self.version = Some(2);
		self.product = Some(1);

		if make_unisex {
			self.make_unisex();
		}

		if let Some(categories) = category_overrides {
			self.categories = categories.clone().into_iter()
				.filter(|c| *c != Category::Pregnant || self.ages.contains(&Age::Adult) || self.ages.contains(&Age::YoungAdult) || self.parts.contains(&Part::Hair))
				.collect();
		}

		if let Some(flags) = flag_overrides {
			let is_default = self.flags.contains(&OutfitFlag::Default);
			self.flags = flags.clone();
			if is_default {
				self.flags.push(OutfitFlag::Default);
			}
		}

		self.cres_index = 0;
		self.shpe_index = 1;
		self.subset_indexes = txmt_ids.iter().enumerate()
			.map(|(j, (subset, _))| SubsetRef {
				shpe_index: 0,
				subset_name: PascalString::new(subset),
				txmt_index: 2 + j as u32
			})
			.collect();
	}

	pub fn age_gender_string(&self) -> String {
		format!("{}{}", Age::stringify(&self.ages), Gender::stringify(&self.genders))
	}
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Category {
	Everyday = 7,
	Swim = 8,
	Pajamas = 16,
	Formal = 32,
	Underwear = 64,
	Skin = 128,
	Pregnant = 256,
	Active = 512,
	TryOn = 1024,
	Overlay = 2048,
	Outerwear = 4096
}

impl Category {
	pub fn from_flag(flag: u32) -> Vec<Self> {
		let mut categories = Vec::new();
		if flag & Self::Everyday as u32 > 0 { categories.push(Self::Everyday) }
		if flag & Self::Swim as u32 > 0 { categories.push(Self::Swim) }
		if flag & Self::Pajamas as u32 > 0 { categories.push(Self::Pajamas) }
		if flag & Self::Formal as u32 > 0 { categories.push(Self::Formal) }
		if flag & Self::Underwear as u32 > 0 { categories.push(Self::Underwear) }
		if flag & Self::Skin as u32 > 0 { categories.push(Self::Skin) }
		if flag & Self::Pregnant as u32 > 0 { categories.push(Self::Pregnant) }
		if flag & Self::Active as u32 > 0 { categories.push(Self::Active) }
		if flag & Self::TryOn as u32 > 0 { categories.push(Self::TryOn) }
		if flag & Self::Overlay as u32 > 0 { categories.push(Self::Overlay) }
		if flag & Self::Outerwear as u32 > 0 { categories.push(Self::Outerwear) }
		categories
	}

	pub fn to_flag(categories: &[Self]) -> u32 {
		categories.iter().map(|c| *c as u32).sum()
	}

	pub fn from_string(s: &str) -> Option<Self> {
		match s {
			"e" | "everyday" => Some(Self::Everyday),
			"f" | "formal" => Some(Self::Formal),
			"u" | "underwear" => Some(Self::Underwear),
			"p" | "pajamas" => Some(Self::Pajamas),
			"s" | "swim" => Some(Self::Swim),
			"a" | "active" => Some(Self::Active),
			"o" | "outerwear" => Some(Self::Outerwear),
			"P" | "pregnant" => Some(Self::Pregnant),
			_ => None
		}
	}

	pub fn all() -> Vec<Self> {
		vec![Self::Everyday, Self::Swim, Self::Pajamas, Self::Formal, Self::Underwear, Self::Pregnant, Self::Active, Self::Outerwear]
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
		ages.iter().map(|a| *a as u32).sum()
	}

	pub fn are_compatible(a: &[Self], b: &[Self]) -> bool {
		(a.len() == 1 && b.len() == 1 && a[0] == b[0]) ||
			(a.contains(&Self::Adult) && (b.contains(&Self::Adult) || b.contains(&Self::YoungAdult))) ||
			(a.contains(&Self::YoungAdult) && (b.contains(&Self::Adult) || b.contains(&Self::YoungAdult))) ||
			(a.contains(&Self::Elder) && b.contains(&Self::Elder))
	}

	pub fn from_string(s: &str) -> Option<Self> {
		match s {
			"p" | "toddler" => Some(Self::Toddler),
			"c" | "child" => Some(Self::Child),
			"t" | "teen" => Some(Self::Teen),
			"y" | "youngadult" => Some(Self::YoungAdult),
			"a" | "adult" => Some(Self::Adult),
			"e" | "elder" => Some(Self::Elder),
			_ => None
		}
	}

	pub fn stringify(ages: &[Self]) -> String {
		let mut age_string = String::new();
		if ages.contains(&Self::Baby) { age_string.push('b'); }
		if ages.contains(&Self::Toddler) { age_string.push('p'); }
		if ages.contains(&Self::Child) { age_string.push('c'); }
		if ages.contains(&Self::Teen) { age_string.push('t'); }
		if ages.contains(&Self::YoungAdult) { age_string.push('y'); }
		if ages.contains(&Self::Adult) { age_string.push('a'); }
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
		genders.iter().map(|g| *g as u32).sum()
	}

	pub fn are_compatible(genders1: &[Self], genders2: &[Self], ages: &[Age]) -> bool {
		ages.contains(&Age::Baby) || ages.contains(&Age::Toddler) || ages.contains(&Age::Child) ||
			(genders1.len() == 1 && genders2.contains(&genders1[0])) ||
			(genders1.len() >= 2 && !genders2.is_empty())
	}

	pub fn from_string(s: &str) -> Vec<Self> {
		match s {
			"f" | "female" => vec![Self::Female],
			"m" | "male" => vec![Self::Male],
			"u" | "unisex" => vec![Self::Female, Self::Male],
			_ => vec![]
		}
	}

	pub fn stringify(genders: &[Self]) -> String {
		if genders.len() > 1 {
			"u".to_string()
		} else if genders.contains(&Self::Male) {
			"m".to_string()
		} else if genders.contains(&Self::Female) {
			"f".to_string()
		} else {
			"".to_string()
		}
	}
}

#[repr(u32)]
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum Shoe {
	#[default]
	None = 0,
	Barefoot = 1,
	Boots = 2,
	Heels = 3,
	Normal = 4,
	Sandals = 5,
	Pajamas = 6,
	Armored = 7
}

impl Shoe {
	pub fn from_flag(flag: u32) -> Self {
		match flag {
			1 => Self::Barefoot,
			2 => Self::Boots,
			3 => Self::Heels,
			4 => Self::Normal,
			5 => Self::Sandals,
			6 => Self::Pajamas,
			7 => Self::Armored,
			_ => Self::None
		}
	}

	pub fn from_string(s: &str) -> Self {
		match s {
			"b" | "barefoot" => Self::Barefoot,
			"B" | "boots" => Self::Boots,
			"h" | "heels" => Self::Heels,
			"d" | "default" | "normal" => Self::Normal,
			"s" | "sandals" => Self::Sandals,
			"p" | "pajamas" => Self::Pajamas,
			"a" | "armor" => Self::Armored,
			_ => Self::None
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
		parts.iter().map(|p| *p as u32).sum()
	}

	pub fn from_string(s: &str) -> Vec<Self> {
		match s {
			"f" | "fullbody" | "body" => vec![Self::Body],
			"t" | "top" => vec![Self::Top],
			"b" | "bottom" => vec![Self::Bottom],
			_ => vec![]
		}
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

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OutfitFlag {
	Hidden = 1,
	Hat = 2,
	Default = 4,
	NoTownies = 8,
	Unused = 16,
	NoEmployees = 32
}

impl OutfitFlag {
	pub fn from_flag(flag: u32) -> Vec<OutfitFlag> {
		let mut outfit_flags = Vec::new();
		if flag & Self::Hidden as u32 > 0 { outfit_flags.push(Self::Hidden) }
		if flag & Self::Hat as u32 > 0 { outfit_flags.push(Self::Hat) }
		if flag & Self::Default as u32 > 0 { outfit_flags.push(Self::Default) }
		if flag & Self::NoTownies as u32 > 0 { outfit_flags.push(Self::NoTownies) }
		if flag & Self::Unused as u32 > 0 { outfit_flags.push(Self::Unused) }
		if flag & Self::NoEmployees as u32 > 0 { outfit_flags.push(Self::NoEmployees) }
		outfit_flags
	}

	pub fn to_flag(outfit_flags: &[Self]) -> u32 {
		outfit_flags.iter().map(|f| *f as u32).sum()
	}

	pub fn from_string(s: &str) -> Option<Self> {
		match s {
			"h" | "hidden" => Some(Self::Hidden),
			"H" | "hat" => Some(Self::Hat),
			"d" | "default" => Some(Self::Default),
			"t" | "notownies" => Some(Self::NoTownies),
			"w" | "noworkers" => Some(Self::NoEmployees),
			_ => None
		}
	}
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum HairTone {
	None,
	Black,
	Brown,
	Blond,
	Red,
	Grey,
	#[default]
	Other
}

impl HairTone {
	pub fn from_pascal_string(pascal_string: &PascalString) -> Self {
		Self::from_string(&pascal_string.to_string())
	}

	pub fn to_pascal_string(self) -> PascalString {
		match self {
			Self::None => PascalString::new("00000000-0000-0000-0000-000000000000"),
			Self::Black => PascalString::new("00000001-0000-0000-0000-000000000000"),
			Self::Brown => PascalString::new("00000002-0000-0000-0000-000000000000"),
			Self::Blond => PascalString::new("00000003-0000-0000-0000-000000000000"),
			Self::Red => PascalString::new("00000004-0000-0000-0000-000000000000"),
			Self::Grey => PascalString::new("00000005-0000-0000-0000-000000000000"),
			Self::Other => PascalString::new("00000006-0000-0000-0000-000000000000")
		}
	}

	pub fn from_string(string: &str) -> Self {
		match string {
			"00000000-0000-0000-0000-000000000000" => Self::None,
			"00000001-0000-0000-0000-000000000000" => Self::Black,
			"00000002-0000-0000-0000-000000000000" => Self::Brown,
			"00000003-0000-0000-0000-000000000000" => Self::Blond,
			"00000004-0000-0000-0000-000000000000" => Self::Red,
			"00000005-0000-0000-0000-000000000000" => Self::Grey,
			_ => Self::Other,
		}
	}
}

#[derive(Clone, Default)]
pub struct SubsetRef {
	pub shpe_index: u32,
	pub subset_name: PascalString,
	pub txmt_index: u32,
}
