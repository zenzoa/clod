use std::error::Error;
use std::io::{ Cursor, Read, Write };

use binrw::BinRead;

use crate::dbpf::{ Identifier, TypeId };
use crate::dbpf::index_entry::IndexEntry;

use crate::dbpf::resource_types::gmdc::Gmdc;
use crate::dbpf::resource_types::gmnd::Gmnd;
use crate::dbpf::resource_types::shpe::Shpe;
use crate::dbpf::resource_types::cres::Cres;

use crate::dbpf::resource_types::txmt::Txmt;
use crate::dbpf::resource_types::txtr::Txtr;

use crate::dbpf::resource_types::gzps::Gzps;
use crate::dbpf::resource_types::idr::Idr;

pub const HEADER_SIZE: usize = 9;

#[derive(Clone)]
pub enum DecodedResource {
	Gmdc(Gmdc),
	Gmnd(Gmnd),
	Shpe(Shpe),
	Cres(Cres),
	Txmt(Txmt),
	Txtr(Txtr),
	Gzps(Gzps),
	Idr(Idr)
}

impl DecodedResource {
	pub fn new(resource: &Resource, title: &str) -> Result<Self, Box<dyn Error>> {
		// println!("READ {}", resource.id);
		match resource.id.type_id {
			TypeId::Gmdc => Ok(DecodedResource::Gmdc(Gmdc::new(resource)?)),
			TypeId::Gmnd => Ok(DecodedResource::Gmnd(Gmnd::new(resource)?)),
			TypeId::Shpe => Ok(DecodedResource::Shpe(Shpe::new(resource)?)),
			TypeId::Cres => Ok(DecodedResource::Cres(Cres::new(resource)?)),
			TypeId::Txmt => Ok(DecodedResource::Txmt(Txmt::new(resource)?)),
			TypeId::Txtr => Ok(DecodedResource::Txtr(Txtr::new(resource)?)),
			TypeId::Gzps => Ok(DecodedResource::Gzps(Gzps::new(resource, title)?)),
			TypeId::Idr => Ok(DecodedResource::Idr(Idr::new(resource)?)),
			_ => Err("Unknown resource type".into())
		}
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		match self {
			Self::Gmdc(gmdc) => { gmdc.to_bytes() }
			Self::Gmnd(gmnd) => { gmnd.to_bytes() }
			Self::Shpe(shpe) => { shpe.to_bytes() }
			Self::Cres(cres) => { cres.to_bytes() }
			Self::Txmt(txmt) => { txmt.to_bytes() }
			Self::Txtr(txtr) => { txtr.to_bytes() }
			Self::Gzps(gzps) => { gzps.to_bytes() }
			Self::Idr(idr) => { idr.to_bytes() }
		}
	}

	pub fn get_id(&self) -> Identifier {
		match self {
			Self::Gmdc(gmdc) => { gmdc.id.clone() }
			Self::Gmnd(gmnd) => { gmnd.id.clone() }
			Self::Shpe(shpe) => { shpe.id.clone() }
			Self::Cres(cres) => { cres.id.clone() }
			Self::Txmt(txmt) => { txmt.id.clone() }
			Self::Txtr(txtr) => { txtr.id.clone() }
			Self::Gzps(gzps) => { gzps.id.clone() }
			Self::Idr(idr) => { idr.id.clone() }
		}
	}

	pub fn to_resource(&self) -> Result<Resource, Box<dyn Error>> {
		Ok(Resource {
			id: self.get_id(),
			data: self.to_bytes()?
		})
	}
}

#[derive(Clone)]
pub struct Resource {
	pub id: Identifier,
	pub data: Vec<u8>
}

impl Resource {
	pub fn read_all(cur: &mut Cursor<&[u8]>, index_entries: &[IndexEntry]) -> Result<Vec<Resource>, Box<dyn Error>> {
		let mut resources = Vec::new();
		for index_entry in index_entries {
			resources.push(Self::read(cur, index_entry)?);
		}
		Ok(resources)
	}

	pub fn read(cur: &mut Cursor<&[u8]>, index_entry: &IndexEntry) -> Result<Self, Box<dyn Error>> {
		cur.set_position(index_entry.resource_offset as u64);
		let mut raw_data = vec![0u8; index_entry.resource_size as usize];
		cur.read_exact(&mut raw_data)?;

		let data = if let Some(uncompressed_size) = Self::get_compression(&raw_data, index_entry) {
			Self::uncompress(&raw_data, index_entry.resource_size, uncompressed_size)?
		} else {
			raw_data
		};

		Ok(Self {
			id: index_entry.id.clone(),
			data
		})
	}

	fn get_compression(data: &[u8], index_entry: &IndexEntry) -> Option<u32> {
		let compression_id = data[5] as u32 * 256 + data[4] as u32;
		if compression_id == 0xfb10 {
			let compressed_size = ((data[3] as u32 * 256 + data[2] as u32) * 256 + data[1] as u32) * 256 + data[0] as u32;
			if compressed_size == index_entry.resource_size {
				let uncompressed_size = (data[6] as u32 * 256 + data[7]as u32) * 256 + data[8] as u32;
				if uncompressed_size > compressed_size {
					return Some(uncompressed_size)
				}
			}
		}
		None
	}

	pub fn uncompress(data: &[u8], compressed_size: u32, uncompressed_size: u32) -> Result<Vec<u8>, Box<dyn Error>> {
		if compressed_size == uncompressed_size {
			return Ok(data[HEADER_SIZE..].to_vec());
		}
		let uncompressed_data = uncompress_data(data, uncompressed_size)?;
		Ok(uncompressed_data)
	}

	pub fn decode(&self, title: &str) -> Result<DecodedResource, Box<dyn Error>> {
		DecodedResource::new(self, title)
	}

	pub fn write(&self, writer: &mut Cursor<Vec<u8>>) -> Result<(), Box<dyn Error>> {
		writer.write(&self.data)?;
		Ok(())
	}
}

fn uncompress_data(data: &[u8], uncompressed_size: u32) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut cur = Cursor::new(data);
	cur.set_position(HEADER_SIZE as u64);

	let mut uncompressed_data = vec![0u8; uncompressed_size as usize];
	let mut pos = 0usize;

	let mut control1 = 0u32;

	while control1 != 0xFC && cur.position() < data.len() as u64 {
		control1 = u8::read(&mut cur)? as u32;
		if cur.position() == data.len() as u64 { break; }

		if control1 <= 0x7F {
			let control2 = u8::read(&mut cur)? as u32;
			if cur.position() == data.len() as u64 { break; }

			let add_length = control1 & 0x03;
			if let Err(_) = copy_plain_text(&mut cur, &mut uncompressed_data, &mut pos, add_length) { break; }

			let copy_length = ((control1 & 0x1C) >> 2) + 3;
			let copy_offset = ((control1 & 0x60) << 3) + control2 + 1;
			if let Err(_) = copy_from_offset(&mut uncompressed_data, &mut pos, copy_offset, copy_length) { break; }

		} else if control1 >= 0x80 && control1 <= 0xBF {
			let control2 = u8::read(&mut cur)? as u32;
			if cur.position() == data.len() as u64 { break; }
			let control3 = u8::read(&mut cur)? as u32;
			if cur.position() == data.len() as u64 { break; }

			let add_length = (control2 >> 6) & 0x03;
			if let Err(_) = copy_plain_text(&mut cur, &mut uncompressed_data, &mut pos, add_length) { break; }

			let copy_length = (control1 & 0x3F) + 4;
			let copy_offset = ((control2 & 0x3F) << 8) + control3 + 1;
			if let Err(_) = copy_from_offset(&mut uncompressed_data, &mut pos, copy_offset, copy_length) { break; }

		} else if control1 >= 0xC0 && control1 <= 0xDF {
			let control2 = u8::read(&mut cur)? as u32;
			if cur.position() == data.len() as u64 { break; }
			let control3 = u8::read(&mut cur)? as u32;
			if cur.position() == data.len() as u64 { break; }
			let control4 = u8::read(&mut cur)? as u32;
			if cur.position() == data.len() as u64 { break; }

			let add_length = control1 & 0x03;
			if let Err(_) = copy_plain_text(&mut cur, &mut uncompressed_data, &mut pos, add_length) { break; }

			let copy_length = ((control1 & 0x0C) << 6)  + control4 + 5;
			let copy_offset = ((control1 & 0x10) << 12) + (control2 << 8) + control3 + 1;
			if let Err(_) = copy_from_offset(&mut uncompressed_data, &mut pos, copy_offset, copy_length) { break; }

		} else if control1 >= 0xE0 && control1 <= 0xFB {
			let add_length = ((control1 & 0x1F) << 2) + 4;
			if let Err(_) = copy_plain_text(&mut cur, &mut uncompressed_data, &mut pos, add_length) { break; }

		} else {
			let add_length = control1 & 0x03;
			if let Err(_) = copy_plain_text(&mut cur, &mut uncompressed_data, &mut pos, add_length) { break; }
		}
	}

	Ok(uncompressed_data)
}

fn copy_plain_text(cur: &mut Cursor<&[u8]>, data: &mut Vec<u8>, pos: &mut usize, length: u32) -> Result<(), Box<dyn Error>> {
	for _ in 0..length {
		let byte = u8::read(cur)?;
		data[*pos] = byte;
		*pos += 1;
		if *pos == data.len() {
			return Err("End of data".into());
		}
	}
	Ok(())
}

fn copy_from_offset(data: &mut Vec<u8>, pos: &mut usize, offset: u32, length: u32) -> Result<(), Box<dyn Error>> {
	let start = *pos - offset as usize;
	for i in 0..length as usize {
		let byte = data[start+i];
		data[*pos] = byte;
		*pos += 1;
		if *pos == data.len() {
			return Err("End of data".into());
		}
	}
	Ok(())
}
