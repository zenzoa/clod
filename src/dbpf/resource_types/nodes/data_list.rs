use std::error::Error;
use std::io::Cursor;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::{ TypeId, PascalString, Transformation, Quaternion };

#[derive(Clone)]
pub struct DataListExtension {
	name: PascalString,
	variables: Vec<DataListVariable>
}

impl DataListExtension {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let block_name = PascalString::read::<u8>(cur)?;
		if &block_name.to_string() != "cDataListExtension" {
			return Err(format!("Invalid cDataListExtension header.").into());
		}
		let _block_id = u32::read_le(cur)?; // expect 0x6a836d56
		let _version = u32::read_le(cur)?; // expect 1

		let _block_name2 = PascalString::read::<u8>(cur)?; // expect "cExtension"
		let _block_id2 = u32::read_le(cur)?; // expect 0
		let _version2 = u32::read_le(cur)?; // expect 3

		let _ext_type = u8::read(cur)?; // expect 7

		let name = PascalString::read::<u8>(cur)?;

		let num_variables = u32::read_le(cur)?;
		let mut variables = Vec::new();
		for _ in 0..num_variables {
			variables.push(DataListVariable::read(cur)?);
		}

		Ok(Self {
			name,
			variables
		})
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		PascalString::new("cDataListExtension").write::<u8>(writer)?;
		(TypeId::DataList as u32).write_le(writer)?;
		1u32.write_le(writer)?;
		PascalString::new("cExtension").write::<u8>(writer)?;
		0u32.write_le(writer)?;
		3u32.write_le(writer)?;
		7u32.write_le(writer)?;

		self.name.write::<u8>(writer)?;

		(self.variables.len() as u32).write_le(writer)?;
		for variable in &self.variables {
			variable.write(writer)?;
		}

		Ok(())
	}
}

#[derive(Clone)]
pub enum DataListVariable {
	Integer(PascalString, u32),
	Float(PascalString, f32),
	Translation(PascalString, Transformation),
	Tag(PascalString, PascalString),
	Array(PascalString, Vec<DataListVariable>),
	Rotation(PascalString, Quaternion),
	Data(PascalString, Vec<u8>),
}

impl DataListVariable {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let var_type = u8::read_le(cur)?;
		let var_name = PascalString::read::<u8>(cur)?;

		match var_type {
			2 => {
				let value = u32::read_le(cur)?;
				Ok(DataListVariable::Integer(var_name, value))
			}
			3 => {
				let value = f32::read_le(cur)?;
				Ok(DataListVariable::Float(var_name, value))
			}
			5 => {
				let value = Transformation::read(cur)?;
				Ok(DataListVariable::Translation(var_name, value))
			}
			6 => {
				let value = PascalString::read::<u8>(cur)?;
				Ok(DataListVariable::Tag(var_name, value))
			}
			7 => {
				let num_variables = u32::read_le(cur)?;
				let mut variables = Vec::new();
				for _ in 0..num_variables {
					variables.push(DataListVariable::read(cur)?);
				}
				Ok(DataListVariable::Array(var_name, variables))
			}
			8 => {
				let value = Quaternion::read(cur)?;
				Ok(DataListVariable::Rotation(var_name, value))
			}
			9 => {
				let data_len = u32::read_le(cur)?;
				let mut data = Vec::new();
				for _ in 0..data_len {
					data.push(u8::read(cur)?);
				}
				Ok(DataListVariable::Data(var_name, data))
			}
			_ => Err("Unknown DataListExtension variable type.".into())
		}
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		match self {
			DataListVariable::Integer(name, value) => {
				name.write::<u8>(writer)?;
				value.write_le(writer)?;
			}
			DataListVariable::Float(name, value) => {
				name.write::<u8>(writer)?;
				value.write_le(writer)?;
			}
			DataListVariable::Translation(name, value) => {
				name.write::<u8>(writer)?;
				value.write(writer)?;
			}
			DataListVariable::Tag(name, value) => {
				name.write::<u8>(writer)?;
				value.write::<u8>(writer)?;
			}
			DataListVariable::Array(name, variables) => {
				name.write::<u8>(writer)?;
				(variables.len() as u32).write_le(writer)?;
				for variable in variables {
					variable.write(writer)?;
				}
			}
			DataListVariable::Rotation(name, value) => {
				name.write::<u8>(writer)?;
				value.write(writer)?;
			}
			DataListVariable::Data(name, data) => {
				name.write::<u8>(writer)?;
				data.write(writer)?;
			}
		}
		Ok(())
	}
}
