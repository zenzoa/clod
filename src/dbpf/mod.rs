use std::error::Error;
use std::io::{ Cursor, Read, Write };
use std::fmt;
use std::convert::From;
use std::fs::{ self, File };
use std::path::Path;

use binrw::{ BinRead, BinWrite };

use crate::dbpf::header::Header;
use crate::dbpf::index_entry::IndexEntry;
use crate::dbpf::resource::{ Resource, DecodedResource };
use crate::dbpf::resource_types::dir::{ Dir, DirItem };

pub mod header;
pub mod index_entry;
pub mod resource;
pub mod resource_types;

#[derive(Clone)]
pub struct Dbpf {
	pub header: Header,
	pub resources: Vec<DecodedResource>,
	pub is_compressed: bool
}

impl Dbpf {
	pub fn new(resources: Vec<DecodedResource>) -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			header: Header::default(),
			resources,
			is_compressed: false
		})
	}

	pub fn read(bytes: &[u8], title: &str) -> Result<Dbpf, Box<dyn Error>> {
		let (resources, header, is_compressed) = Self::read_resources(bytes)?;
		let decoded_resources = resources
			.iter()
			.map(|r| -> Result<DecodedResource, Box<dyn Error>> { r.decode(title) })
			.collect::<Result<Vec<DecodedResource>, Box<dyn Error>>>()?;

		Ok(Dbpf {
			header,
			resources: decoded_resources,
			is_compressed
		})
	}

	pub fn read_resources(bytes: &[u8]) -> Result<(Vec<Resource>, Header, bool), Box<dyn Error>> {
		let mut cur = Cursor::new(bytes);
		let header = Header::read(&mut cur)?;
		cur.set_position(header.index_offset as u64);
		let (index_entries, is_compressed) = IndexEntry::read_all(&mut cur, &header)?;
		let resources = Resource::read_all(&mut cur, &index_entries)?;
		Ok((resources, header, is_compressed))
	}

	pub fn read_from_file(path: &Path, title: &str) -> Result<Dbpf, Box<dyn Error>> {
		let bytes = fs::read(path)?;
		Dbpf::read(&bytes, title)
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>, compress: Option<bool>) -> Result<(), Box<dyn Error>> {
		let resources = self.resources
			.iter()
			.map(|r| -> Result<Resource, Box<dyn Error>> { r.to_resource() })
			.collect::<Result<Vec<Resource>, Box<dyn Error>>>()?;

		let compress = compress.is_some_and(|c| c) || self.is_compressed;
		Self::write_resources(resources, self.header.clone(), writer, compress)
	}

	pub fn write_resources(mut resources: Vec<Resource>, mut header: Header, writer: &mut Cursor<Vec<u8>>, compress: bool) -> Result<(), Box<dyn Error>> {
		if compress {
			let mut dir_items = Vec::new();
			for resource in resources.iter_mut() {
				let uncompressed_size = resource.data.len() as u32;
				if resource.compress()? {
					dir_items.push(DirItem{ id: resource.id.clone(), uncompressed_size })
				}
			}
			if !dir_items.is_empty() {
				let dir = Dir::new(dir_items);
				resources.insert(0, Resource{ id: dir.id.clone(), data: dir.to_bytes()? });
			}
		}

		header.index_entry_count = resources.len() as u32;

		let mut index_entries = Vec::new();
		let mut offset = if header.minor_version >= 1 { 96 } else { 92 };
		for resource in &resources {
			let index_entry = IndexEntry::from_resource(resource, offset);
			index_entries.push(index_entry);
			offset += resource.data.len() as u32;
		}

		header.index_offset = offset;
		let index_entry_size = if header.index_minor_version >= 2 { 24 } else { 20 };
		header.index_size = (index_entries.len() * index_entry_size) as u32;

		header.write(writer)?;

		for resource in &resources {
			resource.write(writer)?;
		}

		writer.set_position(offset as u64);
		for index_entry in &index_entries {
			index_entry.write(writer, header.index_minor_version >= 2)?;
		}

		Ok(())
	}

	pub fn clean_up_resources(&mut self) {
		self.resources.sort_by_key(|res| res.get_id().to_string());
		self.resources.dedup_by_key(|res| res.get_id().to_string());
	}

	pub fn write_to_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
		let mut cur = Cursor::new(Vec::new());
		self.write(&mut cur, None)?;
		let mut new_file = File::create(path)?;
		new_file.write_all(&cur.into_inner())?;
		Ok(())
	}

	pub fn write_package_file(resources: &[DecodedResource], path: &Path, compress: bool) -> Result<(), Box<dyn Error>> {
		let mut new_dbpf = Dbpf::new(resources.to_vec())?;
		new_dbpf.clean_up_resources();
		new_dbpf.is_compressed = compress;
		new_dbpf.write_to_file(path)
	}
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum TypeId {
	#[default]
	Unknown,
	Dir,
	Gmdc,
	Gmnd,
	Shpe,
	Cres,
	Txmt,
	Txtr,
	Gzps,
	Idr,
	Binx,
	Xtol,
	Xhtn,
	Ui,
	Coll,
	TextList,
	DataList,
	BoneData,
	Transform,
	ShapeRef,
	Other(u32)
}

impl From<TypeId> for u32 {
	fn from(type_id: TypeId) -> u32 {
		match type_id {
			TypeId::Unknown => 0xFFFFFFFF,
			TypeId::Dir => 0xE86B1EEF,
			TypeId::Gmdc => 0xAC4F8687,
			TypeId::Gmnd => 0x7BA3838C,
			TypeId::Shpe => 0xFC6EB1F7,
			TypeId::Cres => 0xE519C933,
			TypeId::Txmt => 0x49596978,
			TypeId::Txtr => 0x1C4A276C,
			TypeId::Gzps => 0xEBCF3E27,
			TypeId::Idr => 0xAC506764,
			TypeId::Binx => 0x0C560F39,
			TypeId::Xtol => 0x2C1FD8A1,
			TypeId::Xhtn => 0x8C1580B5,
			TypeId::Ui => 0x00000000,
			TypeId::Coll => 0x6C4F359D,
			TypeId::TextList => 0x53545223,
			TypeId::DataList => 0x6A836D56,
			TypeId::BoneData => 0xE9075BC5,
			TypeId::Transform => 0x65246462,
			TypeId::ShapeRef => 0x65245517,
			TypeId::Other(inner) => inner,
		}
	}
}

impl From<u32> for TypeId {
	fn from(value: u32) -> Self {
		match value {
			0xFFFFFFFF => Self::Unknown,
			0xE86B1EEF => Self::Dir,
			0xAC4F8687 => Self::Gmdc,
			0x7BA3838C => Self::Gmnd,
			0xFC6EB1F7 => Self::Shpe,
			0xE519C933 => Self::Cres,
			0x49596978 => Self::Txmt,
			0x1C4A276C => Self::Txtr,
			0xEBCF3E27 => Self::Gzps,
			0xAC506764 => Self::Idr,
			0x0C560F39 => Self::Binx,
			0x2C1FD8A1 => Self::Xtol,
			0x8C1580B5 => Self::Xhtn,
			0x00000000 => Self::Ui,
			0x6C4F359D => Self::Coll,
			0x53545223 => Self::TextList,
			0x6A836D56 => Self::DataList,
			0xE9075BC5 => Self::BoneData,
			0x65246462 => Self::Transform,
			0x65245517 => Self::ShapeRef,
			inner => Self::Other(inner),
		}
	}
}

impl fmt::Display for TypeId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Unknown => write!(f, "Unknown"),
			Self::Dir => write!(f, "DIR"),
			Self::Gmdc => write!(f, "GMDC"),
			Self::Gmnd => write!(f, "GMND"),
			Self::Shpe => write!(f, "SHPE"),
			Self::Cres => write!(f, "CRES"),
			Self::Txmt => write!(f, "TXMT"),
			Self::Txtr => write!(f, "TXTR"),
			Self::Gzps => write!(f, "GZPS"),
			Self::Idr => write!(f, "3IDR"),
			Self::Binx => write!(f, "BINX"),
			Self::Xtol => write!(f, "XTOL"),
			Self::Xhtn => write!(f, "XHTN"),
			Self::Ui => write!(f, "UI"),
			Self::Coll => write!(f, "COLL"),
			Self::TextList => write!(f, "STR#"),
			Self::DataList => write!(f, "cDataListExtension"),
			Self::BoneData => write!(f, "cBoneDataExtension"),
			Self::Transform => write!(f, "cTransformNode"),
			Self::ShapeRef => write!(f, "cShapeRefNode"),
			Self::Other(inner) => write!(f, "#{inner}"),
		}
	}
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Identifier {
	pub type_id: TypeId,
	pub group_id: u32,
	pub resource_id: u32,
	pub instance_id: u32
}

impl Identifier {
	pub fn new(type_id: u32, group_id: u32, resource_id: u32, instance_id: u32) -> Self {
		Self {
			type_id: TypeId::from(type_id),
			group_id,
			instance_id,
			resource_id
		}
	}

	pub fn read(cur: &mut Cursor<&[u8]>, use_tgir: bool) -> Result<Self, Box<dyn Error>> {
		let type_id = u32::read_le(cur)?;
		let group_id = u32::read_le(cur)?;
		let instance_id = u32::read_le(cur)?;
		let resource_id = if use_tgir { u32::read_le(cur)? } else { 0 };
		Ok(Self::new(type_id, group_id, resource_id, instance_id))
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>, use_tgir: bool) -> Result<(), Box<dyn Error>> {
		u32::from(self.type_id).write_le(writer)?;
		self.group_id.write_le(writer)?;
		self.instance_id.write_le(writer)?;
		if use_tgir {
			self.resource_id.write_le(writer)?;
		}
		Ok(())
	}
}

impl fmt::Display for Identifier {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} {:08X}-{:08X}-{:08X}", self.type_id,  self.group_id, self.resource_id, self.instance_id)
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SevenBitInt(usize);

impl fmt::Display for SevenBitInt {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl SevenBitInt {
	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let mut offset = 0;
		let mut value = 0usize;
		while {
			let byte = u8::read(cur)?;
			value |= (byte as usize & 0x7F) << offset;
			offset += 7;
			byte & 0x80 != 0
		} {}
		Ok(Self(value))
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		let mut value = self.0;
		while {
			let current_byte = value as u8 & 0x7F;
			value >>= 7;
			let has_more = value > 0;
			let byte_to_write = current_byte | if has_more { 0x80 } else { 0 };
			byte_to_write.write(writer)?;
			has_more
		} {}
		Ok(())
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct SevenBitString(Vec<u8>);

impl fmt::Display for SevenBitString {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", String::from_utf8_lossy(&self.0))
	}
}

impl SevenBitString {
	pub fn new(string: &str) -> Self {
		Self(string.as_bytes().to_vec())
	}

	pub fn read(cur: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn Error>> {
		let string_length = SevenBitInt::read(cur)?;
		let mut data = vec![0; string_length.0];
		cur.read_exact(&mut data)?;
		Ok(Self(data))
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		let string_length = SevenBitInt(self.0.len());
		string_length.write(writer)?;
		writer.write_all(&self.0)?;
		Ok(())
	}

	pub fn replace(&self, old: &str, new: &str) -> Self {
		Self::new(&self.to_string().replace(&old, &new))
	}
}

#[derive(Clone, Default, PartialEq)]
pub struct PascalString(Vec<u8>);

impl fmt::Display for PascalString {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", String::from_utf8_lossy(&self.0))
	}
}

impl PascalString {
	pub fn new(string: &str) -> Self {
		Self(string.as_bytes().to_vec())
	}

	pub fn read<T: BinRead + TryInto<usize>>(cur: &mut Cursor<&[u8]>) ->
		Result<Self, Box<dyn Error>>
		where for<'a> <T as BinRead>::Args<'a>: Default {
			let string_length: usize = T::read_le(cur)?.try_into().ok().unwrap();
			let mut data = vec![0; string_length];
			cur.read_exact(&mut data)?;
			Ok(Self(data))
	}

	pub fn write<T: BinWrite + TryFrom<usize>>(&self, writer: &mut Cursor<Vec<u8>>) ->
		Result<(), Box<dyn Error>>
		where for<'a> <T as BinWrite>::Args<'a>: Default {
			let string_length: T = self.0.len().try_into().ok().unwrap();
			string_length.write_le(writer)?;
			writer.write_all(&self.0)?;
			Ok(())
	}
}
