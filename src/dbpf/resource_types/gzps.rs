use std::error::Error;
use std::io::Cursor;

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
	pub hairtone: HairTone,
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

		gzps.version = match cpf.get_prop("version") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		gzps.product = match cpf.get_prop("product") {
			Some(PropertyValue::Uint(val)) => Some(*val),
			_ => None
		};

		gzps.age = match cpf.get_prop("age") {
			Some(PropertyValue::Uint(val)) => Age::from_flag(*val),
			_ => return Err("GZPS is missing \"age\" property.".into())
		};

		gzps.gender = match cpf.get_prop("gender") {
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
			Some(PropertyValue::Uint(val)) => *val,
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

		gzps.category = match cpf.get_prop("category") {
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

		gzps.shape = match cpf.get_prop("shapekeyidx") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"shapekeyidx\" property.".into())
		};

		gzps.resource = match cpf.get_prop("resourcekeyidx") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"resourcekeyidx\" property.".into())
		};

		let num_overrides = match cpf.get_prop("numoverrides") {
			Some(PropertyValue::Uint(val)) => *val,
			_ => return Err("GZPS is missing \"numoverrides\" property.".into())
		};

		for i in 0..num_overrides {
			let shape = match cpf.get_prop(&format!("override{i}shape")) {
				Some(PropertyValue::Uint(val)) => *val,
				_ => return Err(format!("GZPS is missing \"override{i}shape\" property.").into())
			};
			let subset = match cpf.get_prop(&format!("override{i}subset")) {
				Some(PropertyValue::String(val)) => val.clone(),
				_ => return Err(format!("GZPS is missing \"override{i}subset\" property.").into())
			};
			let resource = match cpf.get_prop(&format!("override{i}resourcekeyidx")) {
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
		let mut cur = Cursor::new(Vec::new());

		let mut props = Vec::new();

		if let Some(version) = self.version {
			props.push(("version".to_string(), PropertyValue::Uint(version)));
		}
		if let Some(product) = self.product {
			props.push(("product".to_string(), PropertyValue::Uint(product)));
		}
		props.push(("age".to_string(), PropertyValue::Uint(Age::to_flag(&self.age))));
		props.push(("gender".to_string(), PropertyValue::Uint(Gender::to_flag(&self.gender))));
		props.push(("species".to_string(), PropertyValue::Uint(self.species)));
		props.push(("outfit".to_string(), PropertyValue::Uint(Part::to_flag(&self.outfit))));
		props.push(("parts".to_string(), PropertyValue::Uint(Part::to_flag(&self.parts))));
		props.push(("flags".to_string(), PropertyValue::Uint(self.flags)));
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
		props.push(("category".to_string(), PropertyValue::Uint(Category::to_flag(&self.category))));
		props.push(("shoe".to_string(), PropertyValue::Uint(self.shoe as u32)));
		props.push(("fitness".to_string(), PropertyValue::Uint(self.fitness)));
		props.push(("resourcekeyidx".to_string(), PropertyValue::Uint(self.resource)));
		props.push(("shapekeyidx".to_string(), PropertyValue::Uint(self.shape)));

		props.push(("numoverrides".to_string(), PropertyValue::Uint(self.overrides.len() as u32)));
		for (i, outfit_override) in self.overrides.iter().enumerate() {
			props.push((format!("override{i}shape"), PropertyValue::Uint(outfit_override.shape)));
			props.push((format!("override{i}subset"), PropertyValue::String(outfit_override.subset.clone())));
			props.push((format!("override{i}resourcekeyidx"), PropertyValue::Uint(outfit_override.resource)));
		}

		let cpf = Cpf {
			cpf_type: self.cpf_type,
			version: self.cpf_version,
			props
		};
		cpf.write(&mut cur)?;

		Ok(cur.into_inner())
	}

	pub fn set_property(&mut self, property: &str, value: &str) -> Result<(), Box<dyn Error>> {
		match property {
			"version" => self.version = if value.to_lowercase() == "none" { None } else { Some(value.parse::<u32>()?)},
			"product" => self.product = if value.to_lowercase() == "none" { None } else { Some(value.parse::<u32>()?)},
			"age" => self.age = Age::from_flag(value.parse::<u32>()?),
			"gender" => self.gender = Gender::from_flag(value.parse::<u32>()?),
			"species" => self.species = value.parse::<u32>()?,
			"outfit" => self.outfit = Part::from_flag(value.parse::<u32>()?),
			"parts" => self.parts = Part::from_flag(value.parse::<u32>()?),
			"flags" => self.flags = value.parse::<u32>()?,
			"name" => self.name = PascalString::new(value),
			"creator" => self.creator = PascalString::new(value),
			"family" => self.family = PascalString::new(value),
			"genetic" => self.genetic = if value.to_lowercase() == "none" { None } else { Some(value.parse::<f32>()?)},
			"priority" => self.priority = if value.to_lowercase() == "none" { None } else { Some(value.parse::<u32>()?)},
			"outfit_type" => self.outfit_type = PascalString::new(value),
			"skintone" => self.skintone = PascalString::new(value),
			"hairtone" => self.hairtone = HairTone::from_string(value),
			"category" => self.category = Category::from_flag(value.parse::<u32>()?),
			"shoe" => self.shoe = Shoe::from_flag(value.parse::<u32>()?),
			"fitness" => self.fitness = value.parse::<u32>()?,
			_ => { return Err(format!("No property named '{property}' found in GZPS").into()); }
		}
		Ok(())
	}

	pub fn generate_key(&self) -> String {
		let age = Age::stringify(&self.age, true);
		let gender = Gender::stringify(&self.gender);
		let part = Part::stringify(&self.parts);
		let full_name = self.name.to_string().to_lowercase().trim().to_string();

		let mut name_without_prefix = full_name.clone();
		let re = Regex::new(r"^(?:casie_)?(?:contest_)?[bpctyaeu][mfu](?:body)?(?:bottom)?(?:top)?([a-z,0-9]+)_?").unwrap();
		for (_, [inner]) in re.captures_iter(&full_name).map(|c| c.extract()) {
			name_without_prefix = inner.to_string();
		}

		let mut name_without_ep = name_without_prefix.clone();
		let re2 = Regex::new(r"([a-z,0-9]+)ep\d$").unwrap();
		for (_, [inner]) in re2.captures_iter(&name_without_prefix).map(|c| c.extract()) {
			name_without_ep = inner.to_string();
		}

		format!("{age}{gender}_{part}_{name_without_ep}")
	}

	pub fn hair_name(&self) -> String {
		let age = Age::stringify(&self.age, false);
		let hairtone = if self.hairtone == HairTone::Other { "".to_string() } else { format!("_{}",self.hairtone.stringify()) };
		let hidden = if self.flags & 1 > 0 { "_HIDDEN" } else { "" };
		format!("{}{}{}{}", age, self.hair_group_name(), hairtone, hidden)
	}

	pub fn hair_group_name(&self) -> String {
		let full_name = self.name.to_string().to_lowercase().trim().replace(" ", "");
		let mut group_name = full_name.clone();
		let re = Regex::new(r"^(?:casie_)?y?[bpctyaeu][mfu](?:hair)(.+)").unwrap();
		for (_, [inner]) in re.captures_iter(&full_name).map(|c| c.extract()) {
			group_name = inner.to_string();
		}
		let num_clones = group_name.matches("_clone").count();
		group_name = group_name.replace("_clone", "");
		let hairtone = format!("_{}", self.hairtone.stringify());
		if group_name.contains("santacap") ||
			group_name.contains("mrsclaus") {
				group_name = group_name.replacen(&self.hairtone.stringify(), "", 1);
		} else if group_name.contains("hatballcapup") ||
			group_name.contains("hatbaker_") ||
			group_name.contains("hatfronds") ||
			group_name.contains("hattourguide") ||
			group_name.contains("hatwitch") ||
			group_name.contains("hatwitch") ||
			group_name.contains("masksuperninja") ||
			group_name.contains("ponypuff") ||
			group_name.contains("hatbellhop") ||
			group_name.contains("hatfedoraband") ||
			group_name.contains("hatpanama") {
				group_name = group_name.replacen(&hairtone, "", 1);
		} else {
			group_name = group_name.rsplitn(2, &hairtone).last().unwrap().to_string();
		}
		group_name.push_str(&"_clone".repeat(num_clones));
		group_name.insert_str(0, "hair_");
		group_name.insert_str(0, &Gender::stringify(&self.gender));
		group_name
	}

	pub fn max_resource_key(&self) -> u32 {
		let resource_keys = self.overrides.iter().map(|o| o.resource);
		resource_keys.max().unwrap_or(0)
	}
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Category {
	Everyday = 7,
	Swimwear = 8,
	PJs = 16,
	Formal = 32,
	Undies = 64,
	Skin = 128,
	Maternity = 256,
	Athletic = 512,
	TryOn = 1024,
	Overlay = 2048,
	Outerwear = 4096
}

impl Category {
	pub fn from_flag(flag: u32) -> Vec<Self> {
		let mut categories = Vec::new();
		if flag & Self::Everyday as u32 > 0 { categories.push(Self::Everyday) }
		if flag & Self::Swimwear as u32 > 0 { categories.push(Self::Swimwear) }
		if flag & Self::PJs as u32 > 0 { categories.push(Self::PJs) }
		if flag & Self::Formal as u32 > 0 { categories.push(Self::Formal) }
		if flag & Self::Undies as u32 > 0 { categories.push(Self::Undies) }
		if flag & Self::Skin as u32 > 0 { categories.push(Self::Skin) }
		if flag & Self::Maternity as u32 > 0 { categories.push(Self::Maternity) }
		if flag & Self::Athletic as u32 > 0 { categories.push(Self::Athletic) }
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

	pub fn toggle_category(categories: &mut Vec<Self>, category: Self, value: bool) {
		if value {
			Self::add_category(categories, category);
		} else {
			Self::remove_category(categories, category);
		}
	}

	pub fn add_category(categories: &mut Vec<Self>, category: Self) {
		if !categories.contains(&category) {
			categories.push(category);
		}
	}

	pub fn remove_category(categories: &mut Vec<Self>, category: Self) {
		if let Some(i) = categories.iter().position(|x| *x == category) {
			categories.remove(i);
		}
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

	pub fn toggle_age(ages: &mut Vec<Self>, age: Self, value: bool) {
		if value {
			Self::add_age(ages, age);
		} else {
			Self::remove_age(ages, age);
		}
	}

	pub fn add_age(ages: &mut Vec<Self>, age: Self) {
		if !ages.contains(&age) {
			ages.push(age);
		}
	}

	pub fn remove_age(ages: &mut Vec<Self>, age: Self) {
		if let Some(i) = ages.iter().position(|x| *x == age) {
			ages.remove(i);
		}
	}

	pub fn stringify(ages: &[Self], combine_adults: bool) -> String {
		let mut age_string = String::new();
		if ages.contains(&Self::Baby) { age_string.push('b'); }
		if ages.contains(&Self::Toddler) { age_string.push('p'); }
		if ages.contains(&Self::Child) { age_string.push('c'); }
		if ages.contains(&Self::Teen) { age_string.push('t'); }
		if combine_adults {
			if ages.contains(&Self::Adult) || ages.contains(&Age::YoungAdult) { age_string.push('a'); }
		} else {
			if ages.contains(&Self::Adult) { age_string.push('a'); }
			if ages.contains(&Self::YoungAdult) { age_string.push('y'); }
		}
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

	pub fn toggle_gender(genders: &mut Vec<Self>, gender: Self, value: bool) {
		if value {
			Self::add_gender(genders, gender);
		} else {
			Self::remove_gender(genders, gender);
		}
	}

	pub fn add_gender(genders: &mut Vec<Self>, gender: Self) {
		if !genders.contains(&gender) {
			genders.push(gender);
		}
	}

	pub fn remove_gender(genders: &mut Vec<Self>, gender: Self) {
		if let Some(i) = genders.iter().position(|x| *x == gender) {
			genders.remove(i);
		}
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

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum HairTone {
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
			"00000001-0000-0000-0000-000000000000" => Self::Black,
			"00000002-0000-0000-0000-000000000000" => Self::Brown,
			"00000003-0000-0000-0000-000000000000" => Self::Blond,
			"00000004-0000-0000-0000-000000000000" => Self::Red,
			"00000005-0000-0000-0000-000000000000" => Self::Grey,
			_ => Self::Other,
		}
	}

	pub fn stringify(&self) -> String {
		(match self {
			Self::Black => "black",
			Self::Brown => "brown",
			Self::Blond => "blond",
			Self::Red => "red",
			Self::Grey => "grey",
			Self::Other => "other"
		}).to_string()
	}
}

#[derive(Clone, Default)]
pub struct Override {
	pub shape: u32,
	pub subset: PascalString,
	pub resource: u32,
}
