use std::error::Error;
use std::io::Cursor;
use std::fmt;
use std::collections::HashMap;

use binrw::{ BinRead, BinWrite };

// use xml::reader::{ EventReader, XmlEvent };

use crate::dbpf::PascalString;

#[derive(Clone)]
pub enum PropertyValue {
	Bool(bool),
	Int(u32),
	Float(f32),
	String(PascalString)
}

impl fmt::Display for PropertyValue {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			PropertyValue::Bool(value) => write!(f, "{}", value),
			PropertyValue::Int(value) => write!(f, "{}", value),
			PropertyValue::Float(value) => write!(f, "{}", value),
			PropertyValue::String(value) => write!(f, "{}", value.to_string())
		}
	}
}

pub struct Cpf {
	pub version: u16,
	pub props: HashMap<String, PropertyValue>
}

impl Cpf {
	pub fn read(data: &[u8]) -> Result<Self, Box<dyn Error>> {
		let mut cur = Cursor::new(data);
		let cpf_id = u32::read_le(&mut cur)?;
		if cpf_id == 0xCBE750E0 {
			Self::read_normal(&mut cur)
		} else {
			Self::read_xml(&mut cur)
		}
	}

	pub fn read_normal(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let version = u16::read_le(cur)?;

		let num_props = u32::read_le(cur)?;

		let mut props = HashMap::new();
		for _ in 0..num_props {
			let prop_type = u32::read_le(cur)?;
			let prop_name = PascalString::read::<u32>(cur)?.to_string();
			let prop_value = match prop_type {
				0xCBA908E1 => PropertyValue::Bool(u8::read(cur)? != 0),
				0xEB61E4F7 | 0x0C264712 => PropertyValue::Int(u32::read_le(cur)?),
				0xABC78708 => PropertyValue::Float(f32::read_le(cur)?),
				0x0B8BEA18 => PropertyValue::String(PascalString::read::<u32>(cur)?),
				_ => return Err("Invalid CPF property type.".into())
			};
			props.insert(prop_name, prop_value);
		}

		Ok(Self {
			version,
			props
		})
	}

	pub fn read_xml(_cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		Err("XML not implemented yet.".into())
		// cur.rewind()?;
		// let buf = BufReader::new(cur);
		// let parser = EventReader::new(buf);
		// let mut depth = 0;
		// for e in parser {
		// 	match e {
		// 		Ok(XmlEvent::StartElement { name, .. }) => {
		// 			println!("{:spaces$}+{name}", "", spaces = depth * 2);
		// 			depth += 1;
		// 		}
		// 		Ok(XmlEvent::EndElement { name }) => {
		// 			depth -= 1;
		// 			println!("{:spaces$}-{name}", "", spaces = depth * 2);
		// 		}
		// 		Err(e) => {
		// 			println!("Error: {e}");
		// 			break;
		// 		}
		// 		_ => {}
		// 	}
		// }

		// Ok(Self {
		// 	version: 0,
		// 	props: Vec::new()
		// })
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		0xCBE750E0u32.write_le(writer)?;

		self.version.write_le(writer)?;

		(self.props.len() as u32).write_le(writer)?;

		for (prop_name, prop_value) in self.props.iter() {
			match prop_value {
				PropertyValue::Bool(value) => {
					0xCBA908E1u32.write_le(writer)?;
					PascalString::new(prop_name).write::<u32>(writer)?;
					(if *value { 1u8 } else { 0u8 }).write(writer)?;
				}
				PropertyValue::Int(value) => {
					0xEB61E4F7u32.write_le(writer)?;
					PascalString::new(prop_name).write::<u32>(writer)?;
					value.write_le(writer)?;
				}
				PropertyValue::Float(value) => {
					0xABC78708u32.write_le(writer)?;
					PascalString::new(prop_name).write::<u32>(writer)?;
					value.write_le(writer)?;
				}
				PropertyValue::String(value) => {
					0x0B8BEA18u32.write_le(writer)?;
					PascalString::new(prop_name).write::<u32>(writer)?;
					value.write::<u32>(writer)?;
				}
			}
		}

		Ok(())
	}
}
