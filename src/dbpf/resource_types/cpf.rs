use std::error::Error;
use std::io::Cursor;
use std::fmt;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use binrw::{ BinRead, BinWrite };

use xmltree::{ XMLNode, Element, ParserConfig };

use crate::dbpf::PascalString;

#[derive(Clone, Copy, Default)]
pub enum CpfType {
	#[default]
	Normal,
	XmlUint,
	XmlString
}

pub struct Cpf {
	pub cpf_type: CpfType,
	pub version: Option<u16>,
	pub props: HashMap<String, PropertyValue>
}

impl Cpf {
	pub fn read(data: &[u8]) -> Result<Self, Box<dyn Error>> {
		let mut cur = Cursor::new(data);
		let cpf_id = u32::read_le(&mut cur)?;
		if cpf_id == 0xCBE750E0 {
			Self::read_normal(&mut cur)
		} else {
			cur.set_position(0);
			Self::read_xml(&mut cur)
		}
	}

	pub fn read_normal(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let version = Some(u16::read_le(cur)?);

		let num_props = u32::read_le(cur)?;

		let mut props = HashMap::new();
		for _ in 0..num_props {
			let prop_type = u32::read_le(cur)?;
			let prop_name = PascalString::read::<u32>(cur)?.to_string();
			let prop_value = match DataType::try_from(prop_type) {
				Ok(DataType::Bool) => PropertyValue::Bool(u8::read(cur)? != 0),
				Ok(DataType::Uint) => PropertyValue::Uint(u32::read_le(cur)?),
				Ok(DataType::Int) => PropertyValue::Int(i32::read_le(cur)?),
				Ok(DataType::Float) => PropertyValue::Float(f32::read_le(cur)?),
				Ok(DataType::String) => PropertyValue::String(PascalString::read::<u32>(cur)?),
				_ => return Err("Invalid CPF property type.".into())
			};
			props.insert(prop_name, prop_value);
		}

		Ok(Self {
			cpf_type: CpfType::Normal,
			version,
			props
		})
	}

	pub fn read_xml(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let xml = Element::parse_with_config(
			cur,
			ParserConfig::new()
				.whitespace_to_characters(true)
				.replace_unknown_entity_references(true)
				.add_entity("", ""),
		)?;

		let version = match xml.attributes.get("version") {
			Some(str) => Some(str.parse::<u16>()?),
			None => None
		};

		let cpf_type = match xml.name.as_str() {
			"cGZPropertySetUint32" => CpfType::XmlUint,
			"cGZPropertySetString" => CpfType::XmlString,
			_ => return Err("Invalid CPF XML root tag.".into()),
		};

		let mut props = HashMap::new();
		for child in xml.children {
			if let XMLNode::Element(el) = child {
				let prop_name = match el.attributes.get("key") {
					Some(key) => key.to_string(),
					None => continue
				};

				let name_type = match el.name.as_str() {
					"AnyBoolean" => Some(DataType::Bool),
					"AnyUint32" => Some(DataType::Uint),
					"AnySint32" => Some(DataType::Int),
					"AnyFloat32" => Some(DataType::Float),
					"AnyString" => Some(DataType::String),
					_ => None
				};
				let mut attr_type = None;
				if let Some(attr_string) = el.attributes.get("type") {
					let without_prefix = attr_string.trim_start_matches("0x");
					if let Ok(attr_num) = u32::from_str_radix(without_prefix, 16) {
						if let Ok(attr_type_value) = DataType::try_from(attr_num) {
							attr_type = Some(attr_type_value);
						}
					}
				}
				let prop_type = match (name_type, attr_type) {
					(Some(t1), Some(t2)) => if t1 == t2 { t1 } else { continue },
					(Some(t), None) | (None, Some(t)) => t,
					(None, None) => continue
				};

				let raw_value = el.get_text().unwrap_or("".into());

				let prop_value = match prop_type {
					DataType::Bool => PropertyValue::Bool(
						raw_value == "True"
					),
					DataType::Uint => PropertyValue::Uint(
						match raw_value.strip_prefix("0x") {
							Some(hex) => u32::from_str_radix(hex, 16)?,
							None => u32::from_str(&raw_value)?
						}
					),
					DataType::Int => PropertyValue::Int(
						i32::from_str(&raw_value)?
					),
					DataType::Float => PropertyValue::Float(
						f32::from_str(&raw_value)?
					),
					DataType::String => PropertyValue::String(
						PascalString::new(&raw_value)
					)
				};

				props.insert(prop_name, prop_value);
			}
		}

		Ok(Self {
			cpf_type,
			version,
			props
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		match self.cpf_type {
			CpfType::Normal => self.write_normal(writer),
			CpfType::XmlUint => self.write_xml("cGZPropertySetUint32", writer),
			CpfType::XmlString => self.write_xml("cGZPropertySetString", writer),
		}
	}

	pub fn write_normal(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		0xCBE750E0u32.write_le(writer)?;

		self.version.unwrap_or(0).write_le(writer)?;

		(self.props.len() as u32).write_le(writer)?;

		for (prop_name, prop_value) in self.props.iter() {
			match prop_value {
				PropertyValue::Bool(value) => {
					(DataType::Bool as u32).write_le(writer)?;
					PascalString::new(prop_name).write::<u32>(writer)?;
					(if *value { 1u8 } else { 0u8 }).write(writer)?;
				}
				PropertyValue::Uint(value) => {
					(DataType::Uint as u32).write_le(writer)?;
					PascalString::new(prop_name).write::<u32>(writer)?;
					value.write_le(writer)?;
				}
				PropertyValue::Int(value) => {
					(DataType::Int as u32).write_le(writer)?;
					PascalString::new(prop_name).write::<u32>(writer)?;
					value.write_le(writer)?;
				}
				PropertyValue::Float(value) => {
					(DataType::Float as u32).write_le(writer)?;
					PascalString::new(prop_name).write::<u32>(writer)?;
					value.write_le(writer)?;
				}
				PropertyValue::String(value) => {
					(DataType::String as u32).write_le(writer)?;
					PascalString::new(prop_name).write::<u32>(writer)?;
					value.write::<u32>(writer)?;
				}
			}
		}

		Ok(())
	}

	pub fn write_xml(&self, root_el_name: &str, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		let mut root_el = Element::new(root_el_name);

		if let Some(version) = self.version {
			root_el.attributes.insert("version".to_string(), version.to_string());
		}

		for (prop_name, prop_value) in &self.props {
			let data_type = prop_value.get_data_type();

			let mut prop_el = match data_type {
				DataType::Bool => Element::new("AnyBoolean"),
				DataType::Uint => Element::new("AnyUint32"),
				DataType::Int => Element::new("AnySint32"),
				DataType::Float => Element::new("AnyFloat32"),
				DataType::String => Element::new("AnyString"),
			};

			prop_el.attributes.insert("type".to_string(), format!("0x{:x}", data_type as u32));

			prop_el.attributes.insert("key".to_string(), prop_name.to_string());

			prop_el.children.push(XMLNode::Text(
				match prop_value {
					PropertyValue::Bool(value) => value.to_string(),
					PropertyValue::Uint(value) => value.to_string(),
					PropertyValue::Int(value) => value.to_string(),
					PropertyValue::Float(value) => value.to_string(),
					PropertyValue::String(value) => value.to_string()
				}
			));

			root_el.children.push(XMLNode::Element(prop_el));
		}

		root_el.write(writer)?;

		Ok(())
	}
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
	Bool = 0xCBA908E1,
	Uint = 0xEB61E4F7,
	Int = 0x0C264712,
	Float = 0xABC78708,
	String = 0x0B8BEA18
}

impl TryFrom<u32> for DataType {
	type Error = &'static str;
	fn try_from(value: u32) -> Result<Self, &'static str> {
		match value {
			0xCBA908E1 => Ok(Self::Bool),
			0xEB61E4F7 => Ok(Self::Uint),
			0x0C264712 => Ok(Self::Int),
			0xABC78708 => Ok(Self::Float),
			0x0B8BEA18 => Ok(Self::String),
			 _ => Err("Invalid CPF Data Type"),
		}
	}
}

#[derive(Clone)]
pub enum PropertyValue {
	Bool(bool),
	Uint(u32),
	Int(i32),
	Float(f32),
	String(PascalString)
}

impl fmt::Display for PropertyValue {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			PropertyValue::Bool(value) => write!(f, "{}", value),
			PropertyValue::Uint(value) => write!(f, "{}", value),
			PropertyValue::Int(value) => write!(f, "{}", value),
			PropertyValue::Float(value) => write!(f, "{}", value),
			PropertyValue::String(value) => write!(f, "{}", value)
		}
	}
}

impl PropertyValue {
	pub fn get_data_type(&self) -> DataType {
		match self {
			PropertyValue::Bool(_) => DataType::Bool,
			PropertyValue::Uint(_) => DataType::Uint,
			PropertyValue::Int(_) => DataType::Int,
			PropertyValue::Float(_) => DataType::Float,
			PropertyValue::String(_) => DataType::String
		}
	}
}
