use std::error::Error;

use crate::dbpf::Identifier;
use crate::dbpf::resource::Resource;

#[derive(Clone)]
pub struct Lifo {
	pub id: Identifier,
	pub data: Vec<u8>
}

impl Lifo {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			id: resource.id.clone(),
			data: resource.data.clone()
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		Ok(self.data.clone())
	}
}
