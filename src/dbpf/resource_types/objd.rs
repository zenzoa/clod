use std::error::Error;
use std::io::{ Cursor, Read };

use binrw::{ BinRead, BinWrite };

use crate::dbpf::Identifier;
use crate::dbpf::resource::Resource;

#[derive(Clone)]
pub struct Objd {
	pub id: Identifier,
	pub file_name: Vec<u8>,
	pub version: u32,
	pub default_wall_adjacent: u16,
	pub initial_stack_size: u16,
	pub default_placement: u16,
	pub default_wall_placement: u16,
	pub default_allowed_height: u16,
	pub interaction_table_id: u16,
	pub interaction_group: u16,
	pub object_type: u16,
	pub multi_tile_master_id: u16,
	pub multi_tile_sub_index: u16,
	pub use_default_placement: u16,
	pub look_at_score: u16,
	pub guid: u32,
	pub unlockable: u16,
	pub catalog_use: u16,
	pub price: u16,
	pub body_strings_id: u16,
	pub slot_id: u16,
	pub diagonal_selector_guid: u32,
	pub grid_aligned_selector_guid: u32,
	pub object_ownership: u16,
	pub ignore_globalsim: u16,
	pub cannot_move_out_with: u16,
	pub hauntable: u16,
	pub proxy_guid: u32,
	pub slot_group: u16,
	pub aspiration: u16,
	pub memory_nice: u16,
	pub ignore_quarter_tile_placement: u16,
	pub initial_depreciation: u16,
	pub daily_depreciation: u16,
	pub self_depreciating: u16,
	pub depreciation_limit: u16,
	pub room_sort: u16,
	pub function_sort: u16,
	pub catalog_strings_id: u16,
	pub is_global_sim_object: u16,
	pub tooltip_name_type: u16,
	pub template_version: u16,
	pub niceness_multiplier: u16,
	pub no_duplicate_on_placement: u16,
	pub want_category: u16,
	pub no_new_name_from_template: u16,
	pub object_version: u16,
	pub default_thumbnail_id: u16,
	pub motive_effects_id: u16,
	pub job_object_guid: u32,
	pub catalog_popup_id: u16,
	pub ignore_current_model_index: u16,
	pub level_offset: u16,
	pub shadow_type: u16,
	pub num_attributes: u16,
	pub num_object_arrays: u16,
	pub for_sale_flags: u16,
	pub front_direction: u16,
	pub unused2: u16,
	pub multi_tile_lead: u16,
	pub expansion_flags_1: u16,
	pub expansion_flags_2: u16,
	pub chair_entry_flags: u16,
	pub tile_width: u16,
	pub inhibit_suit_copying: u16,
	pub build_mode_type: u16,
	pub original_guid: u32,
	pub default_graphic: u16,
	pub unused3: u16,
	pub build_mode_subsort: u16,
	pub selector_category: u16,
	pub selector_sub_category: u16,
	pub footprint_mask: u16,
	pub extend_footprint: u16,
	pub object_size: u16,
	pub unused4: u16,
	pub wall_style_sprite_id: u16,
	pub hunger_rating: u16,
	pub comfort_rating: u16,
	pub hygiene_rating: u16,
	pub bladder_rating: u16,
	pub energy_rating: u16,
	pub fun_rating: u16,
	pub room_rating: u16,
	pub gives_skill: u16,
	pub num_type_attributes: u16,
	pub misc_flags: u16,
	pub type_attribute_guid: u32,
	pub function_sub_sort: u16,
	pub downtown_sort: u16,
	pub keep_buying: u16,
	pub vacation_sort: u16,
	pub reset_lot_action: u16,
	pub object_type_3d: u16,
	pub community_sort: u16,
	pub dream_flags: u16,
	pub thumbnail_flags: u16,
	pub scratch_rating: u16,
	pub chew_rating: u16,
	pub unused5: u16,
	pub unused6: u16,
	pub requirements: u16
}

impl Objd {
	pub fn new(resource: &Resource) -> Result<Self, Box<dyn Error>> {
		let mut cur = Cursor::new(&resource.data[..]);

		let mut file_name = vec![0; 64];
		cur.read_exact(&mut file_name)?;

		Ok(Self {
			id: resource.id.clone(),
			file_name,
			version: u32::read_le(&mut cur)?,
			default_wall_adjacent: u16::read_le(&mut cur)?,
			initial_stack_size: u16::read_le(&mut cur)?,
			default_placement: u16::read_le(&mut cur)?,
			default_wall_placement: u16::read_le(&mut cur)?,
			default_allowed_height: u16::read_le(&mut cur)?,
			interaction_table_id: u16::read_le(&mut cur)?,
			interaction_group: u16::read_le(&mut cur)?,
			object_type: u16::read_le(&mut cur)?,
			multi_tile_master_id: u16::read_le(&mut cur)?,
			multi_tile_sub_index: u16::read_le(&mut cur)?,
			use_default_placement: u16::read_le(&mut cur)?,
			look_at_score: u16::read_le(&mut cur)?,
			guid: u32::read_le(&mut cur)?,
			unlockable: u16::read_le(&mut cur)?,
			catalog_use: u16::read_le(&mut cur)?,
			price: u16::read_le(&mut cur)?,
			body_strings_id: u16::read_le(&mut cur)?,
			slot_id: u16::read_le(&mut cur)?,
			diagonal_selector_guid: u32::read_le(&mut cur)?,
			grid_aligned_selector_guid: u32::read_le(&mut cur)?,
			object_ownership: u16::read_le(&mut cur)?,
			ignore_globalsim: u16::read_le(&mut cur)?,
			cannot_move_out_with: u16::read_le(&mut cur)?,
			hauntable: u16::read_le(&mut cur)?,
			proxy_guid: u32::read_le(&mut cur)?,
			slot_group: u16::read_le(&mut cur)?,
			aspiration: u16::read_le(&mut cur)?,
			memory_nice: u16::read_le(&mut cur)?,
			ignore_quarter_tile_placement: u16::read_le(&mut cur)?,
			initial_depreciation: u16::read_le(&mut cur)?,
			daily_depreciation: u16::read_le(&mut cur)?,
			self_depreciating: u16::read_le(&mut cur)?,
			depreciation_limit: u16::read_le(&mut cur)?,
			room_sort: u16::read_le(&mut cur)?,
			function_sort: u16::read_le(&mut cur)?,
			catalog_strings_id: u16::read_le(&mut cur)?,
			is_global_sim_object: u16::read_le(&mut cur)?,
			tooltip_name_type: u16::read_le(&mut cur)?,
			template_version: u16::read_le(&mut cur)?,
			niceness_multiplier: u16::read_le(&mut cur)?,
			no_duplicate_on_placement: u16::read_le(&mut cur)?,
			want_category: u16::read_le(&mut cur)?,
			no_new_name_from_template: u16::read_le(&mut cur)?,
			object_version: u16::read_le(&mut cur)?,
			default_thumbnail_id: u16::read_le(&mut cur)?,
			motive_effects_id: u16::read_le(&mut cur)?,
			job_object_guid: u32::read_le(&mut cur)?,
			catalog_popup_id: u16::read_le(&mut cur)?,
			ignore_current_model_index: u16::read_le(&mut cur)?,
			level_offset: u16::read_le(&mut cur)?,
			shadow_type: u16::read_le(&mut cur)?,
			num_attributes: u16::read_le(&mut cur)?,
			num_object_arrays: u16::read_le(&mut cur)?,
			for_sale_flags: u16::read_le(&mut cur)?,
			front_direction: u16::read_le(&mut cur)?,
			unused2: u16::read_le(&mut cur)?,
			multi_tile_lead: u16::read_le(&mut cur)?,
			expansion_flags_1: u16::read_le(&mut cur)?,
			expansion_flags_2: u16::read_le(&mut cur)?,
			chair_entry_flags: u16::read_le(&mut cur)?,
			tile_width: u16::read_le(&mut cur)?,
			inhibit_suit_copying: u16::read_le(&mut cur)?,
			build_mode_type: u16::read_le(&mut cur)?,
			original_guid: u32::read_le(&mut cur)?,
			default_graphic: u16::read_le(&mut cur)?,
			unused3: u16::read_le(&mut cur)?,
			build_mode_subsort: u16::read_le(&mut cur)?,
			selector_category: u16::read_le(&mut cur)?,
			selector_sub_category: u16::read_le(&mut cur)?,
			footprint_mask: u16::read_le(&mut cur)?,
			extend_footprint: u16::read_le(&mut cur)?,
			object_size: u16::read_le(&mut cur)?,
			unused4: u16::read_le(&mut cur)?,
			wall_style_sprite_id: u16::read_le(&mut cur)?,
			hunger_rating: u16::read_le(&mut cur)?,
			comfort_rating: u16::read_le(&mut cur)?,
			hygiene_rating: u16::read_le(&mut cur)?,
			bladder_rating: u16::read_le(&mut cur)?,
			energy_rating: u16::read_le(&mut cur)?,
			fun_rating: u16::read_le(&mut cur)?,
			room_rating: u16::read_le(&mut cur)?,
			gives_skill: u16::read_le(&mut cur)?,
			num_type_attributes: u16::read_le(&mut cur)?,
			misc_flags: u16::read_le(&mut cur)?,
			type_attribute_guid: u32::read_le(&mut cur)?,
			function_sub_sort: u16::read_le(&mut cur)?,
			downtown_sort: u16::read_le(&mut cur)?,
			keep_buying: u16::read_le(&mut cur)?,
			vacation_sort: u16::read_le(&mut cur)?,
			reset_lot_action: u16::read_le(&mut cur)?,
			object_type_3d: u16::read_le(&mut cur)?,
			community_sort: u16::read_le(&mut cur)?,
			dream_flags: u16::read_le(&mut cur)?,
			thumbnail_flags: u16::read_le(&mut cur)?,
			scratch_rating: u16::read_le(&mut cur)?,
			chew_rating: u16::read_le(&mut cur)?,
			unused5: u16::read_le(&mut cur)?,
			unused6: u16::read_le(&mut cur)?,
			requirements: u16::read_le(&mut cur)?
		})
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		let mut cur = Cursor::new(Vec::new());

		self.file_name.write(&mut cur)?;
		self.version.write_le(&mut cur)?;
		self.default_wall_adjacent.write_le(&mut cur)?;
		self.initial_stack_size.write_le(&mut cur)?;
		self.default_placement.write_le(&mut cur)?;
		self.default_wall_placement.write_le(&mut cur)?;
		self.default_allowed_height.write_le(&mut cur)?;
		self.interaction_table_id.write_le(&mut cur)?;
		self.interaction_group.write_le(&mut cur)?;
		self.object_type.write_le(&mut cur)?;
		self.multi_tile_master_id.write_le(&mut cur)?;
		self.multi_tile_sub_index.write_le(&mut cur)?;
		self.use_default_placement.write_le(&mut cur)?;
		self.look_at_score.write_le(&mut cur)?;
		self.guid.write_le(&mut cur)?;
		self.unlockable.write_le(&mut cur)?;
		self.catalog_use.write_le(&mut cur)?;
		self.price.write_le(&mut cur)?;
		self.body_strings_id.write_le(&mut cur)?;
		self.slot_id.write_le(&mut cur)?;
		self.diagonal_selector_guid.write_le(&mut cur)?;
		self.grid_aligned_selector_guid.write_le(&mut cur)?;
		self.object_ownership.write_le(&mut cur)?;
		self.ignore_globalsim.write_le(&mut cur)?;
		self.cannot_move_out_with.write_le(&mut cur)?;
		self.hauntable.write_le(&mut cur)?;
		self.proxy_guid.write_le(&mut cur)?;
		self.slot_group.write_le(&mut cur)?;
		self.aspiration.write_le(&mut cur)?;
		self.memory_nice.write_le(&mut cur)?;
		self.ignore_quarter_tile_placement.write_le(&mut cur)?;
		self.initial_depreciation.write_le(&mut cur)?;
		self.daily_depreciation.write_le(&mut cur)?;
		self.self_depreciating.write_le(&mut cur)?;
		self.depreciation_limit.write_le(&mut cur)?;
		self.room_sort.write_le(&mut cur)?;
		self.function_sort.write_le(&mut cur)?;
		self.catalog_strings_id.write_le(&mut cur)?;
		self.is_global_sim_object.write_le(&mut cur)?;
		self.tooltip_name_type.write_le(&mut cur)?;
		self.template_version.write_le(&mut cur)?;
		self.niceness_multiplier.write_le(&mut cur)?;
		self.no_duplicate_on_placement.write_le(&mut cur)?;
		self.want_category.write_le(&mut cur)?;
		self.no_new_name_from_template.write_le(&mut cur)?;
		self.object_version.write_le(&mut cur)?;
		self.default_thumbnail_id.write_le(&mut cur)?;
		self.motive_effects_id.write_le(&mut cur)?;
		self.job_object_guid.write_le(&mut cur)?;
		self.catalog_popup_id.write_le(&mut cur)?;
		self.ignore_current_model_index.write_le(&mut cur)?;
		self.level_offset.write_le(&mut cur)?;
		self.shadow_type.write_le(&mut cur)?;
		self.num_attributes.write_le(&mut cur)?;
		self.num_object_arrays.write_le(&mut cur)?;
		self.for_sale_flags.write_le(&mut cur)?;
		self.front_direction.write_le(&mut cur)?;
		self.unused2.write_le(&mut cur)?;
		self.multi_tile_lead.write_le(&mut cur)?;
		self.expansion_flags_1.write_le(&mut cur)?;
		self.expansion_flags_2.write_le(&mut cur)?;
		self.chair_entry_flags.write_le(&mut cur)?;
		self.tile_width.write_le(&mut cur)?;
		self.inhibit_suit_copying.write_le(&mut cur)?;
		self.build_mode_type.write_le(&mut cur)?;
		self.original_guid.write_le(&mut cur)?;
		self.default_graphic.write_le(&mut cur)?;
		self.unused3.write_le(&mut cur)?;
		self.build_mode_subsort.write_le(&mut cur)?;
		self.selector_category.write_le(&mut cur)?;
		self.selector_sub_category.write_le(&mut cur)?;
		self.footprint_mask.write_le(&mut cur)?;
		self.extend_footprint.write_le(&mut cur)?;
		self.object_size.write_le(&mut cur)?;
		self.unused4.write_le(&mut cur)?;
		self.wall_style_sprite_id.write_le(&mut cur)?;
		self.hunger_rating.write_le(&mut cur)?;
		self.comfort_rating.write_le(&mut cur)?;
		self.hygiene_rating.write_le(&mut cur)?;
		self.bladder_rating.write_le(&mut cur)?;
		self.energy_rating.write_le(&mut cur)?;
		self.fun_rating.write_le(&mut cur)?;
		self.room_rating.write_le(&mut cur)?;
		self.gives_skill.write_le(&mut cur)?;
		self.num_type_attributes.write_le(&mut cur)?;
		self.misc_flags.write_le(&mut cur)?;
		self.type_attribute_guid.write_le(&mut cur)?;
		self.function_sub_sort.write_le(&mut cur)?;
		self.downtown_sort.write_le(&mut cur)?;
		self.keep_buying.write_le(&mut cur)?;
		self.vacation_sort.write_le(&mut cur)?;
		self.reset_lot_action.write_le(&mut cur)?;
		self.object_type_3d.write_le(&mut cur)?;
		self.community_sort.write_le(&mut cur)?;
		self.dream_flags.write_le(&mut cur)?;
		self.thumbnail_flags.write_le(&mut cur)?;
		self.scratch_rating.write_le(&mut cur)?;
		self.chew_rating.write_le(&mut cur)?;
		self.unused5.write_le(&mut cur)?;
		self.unused6.write_le(&mut cur)?;
		self.requirements.write_le(&mut cur)?;

		let mut file_name_2 = Vec::new();
		for c in &self.file_name {
			if *c == 0 {
				break;
			} else {
				file_name_2.push(*c);
			}
		}
		(file_name_2.len() as u32).write_le(&mut cur)?;
		file_name_2.write(&mut cur)?;

		Ok(cur.into_inner())
	}

	pub fn function_sort_string(&self) -> String {
		match self.function_sort {
			0x1 => match self.function_sub_sort {
				0x1 => "Seating_DiningChairs".to_string(),
				0x2 =>"Seating_Armchairs".to_string(),
				0x4 => "Seating_Sofas".to_string(),
				0x8 => "Seating_Beds".to_string(),
				0x10 => "Seating_Recliners".to_string(),
				0x80 => "Seating_Misc".to_string(),
				_ => "Seating_Unknown".to_string()
			},
			0x2 => match self.function_sub_sort {
				0x1 => "Surfaces_Counters".to_string(),
				0x2 => "Surfaces_Tables".to_string(),
				0x4 => "Surfaces_EndTables".to_string(),
				0x8 => "Surfaces_Desks".to_string(),
				0x10 => "Surfaces_CoffeeTables".to_string(),
				0x20 => "Surfaces_Shelfs".to_string(),
				0x80 => "Surfaces_Misc".to_string(),
				_ => "Surfaces_Unknown".to_string()
			},
			0x4 => match self.function_sub_sort {
				0x1 => "Appliances_Cooking".to_string(),
				0x2 => "Appliances_Refrigerators".to_string(),
				0x4 => "Appliances_Small".to_string(),
				0x8 => "Appliances_Large".to_string(),
				0x80 => "Appliances_Misc".to_string(),
				_ => "Appliances_Unknown".to_string()
			},
			0x8 => match self.function_sub_sort {
				0x1 => "Electronics_Entertainment".to_string(),
				0x2 => "Electronics_TVComputers".to_string(),
				0x4 => "Electronics_Audio".to_string(),
				0x8 => "Electronics_Small".to_string(),
				0x80 => "Electronics_Misc".to_string(),
				_ => "Electronics_Unknown".to_string()
			},
			0x10 => match self.function_sub_sort {
				0x1 => "Plumbing_Toilets".to_string(),
				0x2 => "Plumbing_Showers".to_string(),
				0x4 => "Plumbing_Sinks".to_string(),
				0x8 => "Plumbing_HotTubs".to_string(),
				0x80 => "Plumbing_Misc".to_string(),
				_ => "Plumbing_Unknown".to_string()
			},
			0x20 => match self.function_sub_sort {
				0x1 => "Decorative_Wall".to_string(),
				0x2 => "Decorative_Sculpture".to_string(),
				0x4 => "Decorative_Rugs".to_string(),
				0x8 => "Decorative_Plants".to_string(),
				0x10 => "Decorative_Mirrors".to_string(),
				0x20 => "Decorative_Curtains".to_string(),
				0x80 => "Decorative_Misc".to_string(),
				_ => "Decorative_Unknown".to_string()
			},
			0x40 => match self.function_sub_sort {
				0x2 => "General_Dressers".to_string(),
				0x8 => "General_Party".to_string(),
				0x10 => "General_Children".to_string(),
				0x20 => "General_Cars".to_string(),
				0x40 => "General_Pets".to_string(),
				0x80 => "General_Misc".to_string(),
				_ => "General_Unknown".to_string()
			},
			0x80 => match self.function_sub_sort {
				0x1 => "Lighting_TableLamps".to_string(),
				0x2 => "Lighting_FloorLamps".to_string(),
				0x4 => "Lighting_WallLamps".to_string(),
				0x8 => "Lighting_CeilingLamps".to_string(),
				0x10 => "Lighting_Outdoor".to_string(),
				0x80 => "Lighting_Misc".to_string(),
				_ => "Lighting_Unknown".to_string()
			},
			0x100 => match self.function_sub_sort {
				0x1 => "Hobbies_Creative".to_string(),
				0x2 => "Hobbies_Knowledge".to_string(),
				0x4 => "Hobbies_Exercise".to_string(),
				0x8 => "Hobbies_Recreation".to_string(),
				0x80 => "Hobbies_Misc".to_string(),
				_ => "Hobbies_Unknown".to_string(),
			},
			0x400 => "AspirationRewards".to_string(),
			0x800 => "CareerRewards".to_string(),
			_ => {
				if self.community_sort != 0 {
					"Community".to_string()
				} else if self.room_sort == 0 {
					match self.build_mode_type {
						0x1 => match self.build_mode_subsort{
							0x8 => "Build_Columns".to_string(),
							0x20 => "Build_Staircases".to_string(),
							0x40 => "Build_Pools".to_string(),
							0x100 => "Build_TwoStoryColumns".to_string(),
							0x200 => "Build_ConnectingColumns".to_string(),
							0x400 => "Build_Garages".to_string(),
							0x800 => "Build_Elevators".to_string(),
							0x1000 => "Build_Architecture".to_string(),
							_ => "Unknown".to_string()
						},
						0x4 => match self.build_mode_subsort{
							0x1 => "Build_Trees".to_string(),
							0x2 => "Build_Shrubs".to_string(),
							0x4 => "Build_Flowers".to_string(),
							0x10 => "Build_Gardening".to_string(),
							_ => "Unknown".to_string()
						},
						0x8 => match self.build_mode_subsort{
							0x1 => "Build_Doors".to_string(),
							0x2 => "Build_TwoStoryWindows".to_string(),
							0x4 => "Build_Windows".to_string(),
							0x8 => "Build_Gates".to_string(),
							0x10 => "Build_Arches".to_string(),
							0x100 => "Build_TwoStoryDoors".to_string(),
							_ => "Unknown".to_string()
						},
						_ => "Unknown".to_string()
					}
				} else {
					"Unknown".to_string()
				}
			}
		}
	}
}
