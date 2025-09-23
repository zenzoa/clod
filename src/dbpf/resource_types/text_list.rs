use std::error::Error;
use std::io::{ Cursor, Read };
use std::string::FromUtf8Error;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::Identifier;
use crate::dbpf::resource::Resource;

#[derive(Clone)]
pub struct TextList {
	pub id: Identifier,
	pub key_name: [u8;64],
	pub strings: Vec<StringItem>
}

impl TextList {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let mut cur = Cursor::new(&resource.data[..]);

		let mut key_name = [0u8; 64];
		cur.read_exact(&mut key_name)?;

		let format = u16::read_le(&mut cur)?;
		if format != 0xfffd {
			return Err("STR# has invalid data format".into());
		}

		let num_strings = u16::read_le(&mut cur)?;

		let mut strings = Vec::new();
		for _ in 0..num_strings {
			let string_item = StringItem::new(&mut cur)?;
			strings.push(string_item);
		}

		Ok(Self {
			id: resource.id.clone(),
			key_name,
			strings
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let bytes: Vec<u8> = Vec::new();
		let mut cur = Cursor::new(bytes);

		self.key_name.write(&mut cur)?;

		0xfffdu16.write_le(&mut cur)?;

		(self.strings.len() as u16).write_le(&mut cur)?;

		for string_item in &self.strings {
			string_item.to_bytes()?.write(&mut cur)?;
		}

		Ok(cur.into_inner())
	}

	pub fn create_empty(id: Identifier) -> Self {
		Self {
			id,
			key_name: [0; 64],
			strings: vec![
				StringItem {
					language_code: 0x01,
					title: "".to_string(),
					description: "".to_string()
				}
			]

		}
	}
}

#[derive(Clone)]
pub struct StringItem {
	pub language_code: u8,
	pub title: String,
	pub description: String
}

impl StringItem {
	pub fn new(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			language_code: u8::read(cur)?,
			title: read_null_terminating_string(cur)?,
			description: read_null_terminating_string(cur)?
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let bytes: Vec<u8> = Vec::new();
		let mut cur = Cursor::new(bytes);

		self.language_code.write(&mut cur)?;

		self.title.as_bytes().write(&mut cur)?;
		0u8.write(&mut cur)?;

		self.description.as_bytes().write(&mut cur)?;
		0u8.write(&mut cur)?;

		Ok(cur.into_inner())
	}
}

fn read_null_terminating_string(cur: &mut Cursor<&[u8]>) -> Result<String, FromUtf8Error> {
	let mut bytes = Vec::new();
	while let Ok(byte) = u8::read(cur) {
		if byte == 0 {
			break;
		} else {
			bytes.push(byte);
		}
	}
	String::from_utf8(bytes)
}
