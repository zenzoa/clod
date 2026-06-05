use std::error::Error;
use std::io::Cursor;
use std::fmt;
use std::convert::TryFrom;
use std::str::FromStr;

use binrw::{ BinRead, BinWrite };

use xmltree::{ XMLNode, Element, ParserConfig };

use crate::dbpf::PascalString;

#[derive(Clone)]
pub struct Cpf {
	pub props: Vec<(String, PropertyValue)>
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
		let _version = u16::read_le(cur)?;

		let num_props = u32::read_le(cur)?;

		let mut props = Vec::new();
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
			props.push((prop_name, prop_value));
		}

		Ok(Self {
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

		let _version = match xml.attributes.get("version") {
			Some(str) => Some(str.parse::<u16>()?),
			None => None
		};

		let _cpf_type = xml.name.as_str();

		let mut props = Vec::new();
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
							None => i64::from_str(&raw_value)? as u32
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

				props.push((prop_name, prop_value));
			}
		}

		Ok(Self {
			props
		})
	}

	pub fn write_props(props: &[(String, PropertyValue)], writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		0xCBE750E0u32.write_le(writer)?;

		2u16.write_le(writer)?;

		(props.len() as u32).write_le(writer)?;

		for (prop_name, prop_value) in props.iter() {
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

	pub fn get_prop(&self, key: &str) -> Option<&PropertyValue> {
		for prop in &self.props {
			if prop.0 == key {
				return Some(&prop.1);
			}
		}
		None
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

#[derive(Clone, Debug)]
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
