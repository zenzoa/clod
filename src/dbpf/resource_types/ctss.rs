use std::error::Error;

use crate::dbpf::Identifier;
use crate::dbpf::resource::Resource;

use super::text_list::TextList;

#[derive(Clone)]
pub struct Ctss {
	pub id: Identifier,
	pub text_list: TextList
}

impl Ctss {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			id: resource.id.clone(),
			text_list: TextList::new(resource)?
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		self.text_list.to_bytes()
	}
}
