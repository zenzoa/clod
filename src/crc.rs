use crc_any::CRC;

pub fn hash_crc24(string: &str) -> u32 {
	let mut crc24 = CRC::create_crc(0x01864CFB, 24, 0x00B704CE, 0, false);
	let lowercase_string = string.trim().to_lowercase();
	crc24.digest(lowercase_string.as_bytes());
	crc24.get_crc() as u32 | 0xFF000000
}

pub fn hash_crc32(string: &str) -> u32 {
	let mut crc32 = CRC::create_crc(0x04C11DB7, 32, 0xFFFFFFFF, 0, false);
	let lowercase_string = string.trim().to_lowercase();
	crc32.digest(lowercase_string.as_bytes());
	crc32.get_crc() as u32
}
