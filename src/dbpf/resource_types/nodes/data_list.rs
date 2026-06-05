use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::SevenBitString;


#[derive(Clone)]
pub struct DataListNode {
	pub title: SevenBitString,
	pub items: Vec<DataListNodeItem>
}

impl DataListNode {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let block_name = SevenBitString::read(cur)?;
		if &block_name.to_string() != "cDataListExtension" {
			return Err("Invalid cDataListExtension header.".into());
		}

		let _block_id = u32::read_le(cur)?; // expect 0x6a836d56
		let _block_version = u32::read_le(cur)?; // expect 1
		let _extension_name = SevenBitString::read(cur)?; // expect "cExtension"
		let _class_id = u32::read_le(cur)?; // expect 0
		let _class_version = u32::read_le(cur)?; // expect 3
		let _extension_type = u8::read(cur)?; // expect 7

		let title = SevenBitString::read(cur)?;

		let items = DataListNodeItem::read_list(cur)?;

		Ok(Self {
			title,
			items
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		SevenBitString::new("cDataListExtension").write(writer)?;
		0x6a836d56u32.write_le(writer)?;
		1u32.write_le(writer)?;
		SevenBitString::new("cExtension").write(writer)?;
		0u32.write_le(writer)?;
		3u32.write_le(writer)?;
		7u8.write(writer)?;
		self.title.write(writer)?;
		DataListNodeItem::write_list(&self.items, writer)?;
		Ok(())
	}
}

#[derive(Clone)]
pub struct DataListNodeItem {
	pub name: SevenBitString,
	pub value: DataListNodeItemValue
}

#[derive(Clone)]
pub enum DataListNodeItemValue {
	Integer(u32),
	Float(f32),
	Translation((f32, f32, f32)),
	String(SevenBitString),
	Array(Vec<DataListNodeItem>),
	Rotation((f32, f32, f32, f32)),
	Data(Vec<u8>),
}

impl DataListNodeItem {
	pub fn read_list(cur: &mut Cursor<&[u8]>) -> Result<Vec<Self>, Box<dyn Error>> {
		let item_count = u32::read_le(cur)?;
		let mut items = Vec::new();
		for _ in 0..item_count {
			items.push(Self::read(cur)?);
		}
		Ok(items)
	}

	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let item_type = u8::read(cur)?;
		let name = SevenBitString::read(cur)?;
		let value = match item_type {
			2 => {
				let v = u32::read_le(cur)?;
				DataListNodeItemValue::Integer(v)
			}
			3 => {
				let v = f32::read_le(cur)?;
				DataListNodeItemValue::Float(v)
			}
			5 => {
				let v1 = f32::read_le(cur)?;
				let v2 = f32::read_le(cur)?;
				let v3 = f32::read_le(cur)?;
				DataListNodeItemValue::Translation((v1, v2, v3))
			}
			6 => {
				let item_value = SevenBitString::read(cur)?;
				DataListNodeItemValue::String(item_value)
			}
			7 => {
				let array_items = Self::read_list(cur)?;
				DataListNodeItemValue::Array(array_items)
			}
			8 => {
				let item_value1 = f32::read_le(cur)?;
				let item_value2 = f32::read_le(cur)?;
				let item_value3 = f32::read_le(cur)?;
				let item_value4 = f32::read_le(cur)?;
				DataListNodeItemValue::Rotation((item_value1, item_value2, item_value3, item_value4))
			}
			9 => {
				let data_length = u32::read_le(cur)?;
				let mut data = Vec::new();
				for _ in 0..data_length {
					data.push(u8::read(cur)?);
				}
				DataListNodeItemValue::Data(data)
			}
			_ => {
				return Err("Unknown Data List Extension item found".into())
			}
		};

		Ok(Self {
			name,
			value
		})
	}

	pub fn write_list(item_list: &[DataListNodeItem], writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		(item_list.len() as u32).write_le(writer)?;
		for item in item_list {
			item.write(writer)?;
		}
		Ok(())
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		match &self.value {
			DataListNodeItemValue::Integer(v) => {
				2u32.write_le(writer)?;
				self.name.write(writer)?;
				v.write_le(writer)?;
			}
			DataListNodeItemValue::Float(v) => {
				3u32.write_le(writer)?;
				self.name.write(writer)?;
				v.write_le(writer)?;
			}
			DataListNodeItemValue::Translation((v1, v2, v3)) => {
				5u32.write_le(writer)?;
				self.name.write(writer)?;
				v1.write_le(writer)?;
				v2.write_le(writer)?;
				v3.write_le(writer)?;
			}
			DataListNodeItemValue::String(v) => {
				6u32.write_le(writer)?;
				self.name.write(writer)?;
				v.write(writer)?;
			}
			DataListNodeItemValue::Array(v) => {
				7u32.write_le(writer)?;
				self.name.write(writer)?;
				DataListNodeItem::write_list(v, writer)?;
			}
			DataListNodeItemValue::Rotation((v1, v2, v3, v4)) => {
				8u32.write_le(writer)?;
				self.name.write(writer)?;
				v1.write_le(writer)?;
				v2.write_le(writer)?;
				v3.write_le(writer)?;
				v4.write_le(writer)?;
			}
			DataListNodeItemValue::Data(v) => {
				9u32.write_le(writer)?;
				self.name.write(writer)?;
				v.write(writer)?;
			}
		}
		Ok(())
	}
}
