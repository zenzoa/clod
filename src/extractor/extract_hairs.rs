use std::error::Error;
use std::path::{ Path, PathBuf };
use std::collections::HashMap;

use crate::dbpf::Dbpf;
use crate::dbpf::resource::DecodedResource;
use crate::dbpf::resource_types::gzps::{ Gzps, Age, Category, Part, HairTone };
use crate::dbpf::resource_types::idr::Idr;

use super::{ get_skin_packages, create_folder };

#[derive(Clone)]
struct Hair {
	gzps: Gzps,
	idr: Idr
}

pub fn extract_hairs(input_path: Option<PathBuf>, output_path: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
	let input_path = input_path.unwrap_or(PathBuf::from("./"));
	let output_path = output_path.unwrap_or(input_path.clone());

	let all_hairs = get_hairs(&input_path)?;
	let mut hairs_by_family: HashMap<String, Vec<Hair>> = HashMap::new();
	for (_, hair) in all_hairs {
		let family = hair.gzps.family.to_string();
		match hairs_by_family.get_mut(&family) {
			Some(hairs) => { hairs.push(hair.clone()); },
			None => { hairs_by_family.insert(family, vec![hair.clone()]); }
		}
	}

	for (family, hairs) in hairs_by_family.iter_mut() {
		let is_hat = hairs.iter().any(|h| h.gzps.flags & 2 > 0);

		let adult_youngadult_combined = hairs.iter()
			.any(|h| h.gzps.age.contains(&Age::Adult) && h.gzps.age.contains(&Age::YoungAdult));

		let mut hat_ages = Vec::new();
		for hair in hairs.iter() {
			if hair.gzps.flags & 2 > 0 {
				for age in &hair.gzps.age {
					if !hat_ages.contains(&age) {
						hat_ages.push(age);
					}
				}
			}
		}

		let is_special_color = hairs.iter()
			.any(|h| h.gzps.hairtone == HairTone::Other && !h.gzps.name.to_string().contains("bald_bare"));

		let group_name = lookup_group(family);

		let folder_path = create_folder(&output_path, &group_name)?;

		for hair in hairs.iter() {
			let hair_name = hair.gzps.hair_name();

			let duplicates_combined_age = adult_youngadult_combined &&
				(hair.gzps.age.contains(&Age::Adult) || hair.gzps.age.contains(&Age::YoungAdult)) &&
				(hair.gzps.age.len() == 1);

			let mut duplicates_hat_age = false;
			if is_hat && hair.gzps.flags & 2 == 0 && !hair_name.contains("santacap") {
				for age in &hair.gzps.age {
					if hat_ages.contains(&age) {
						duplicates_hat_age = true;
						break;
					}
				}
			}
			if hair_name.contains("_nohat_") {
				duplicates_hat_age = true;
			}

			let wrong_color = (is_special_color && hair.gzps.hairtone != HairTone::Other) ||
				(!is_special_color && hair.gzps.hairtone == HairTone::Other);

			let extension = if duplicates_combined_age || duplicates_hat_age || wrong_color {
				".package.off"
			} else {
				".package"
			};
			let file_name = format!("{}{}", hair_name, extension);

			let file_path = if group_name.starts_with("fhair_hatmascotknight") && hair_name.contains("mhair_pompodore") {
				let alt_folder = create_folder(&output_path, &group_name.replace("fhair", "mhair"))?;
				alt_folder.join(file_name)

			} else if hair_name.contains("fhair_hatballcapnpc_fastfood") {
				let alt_folder = create_folder(&output_path, "fhair_hatballcapnpc_fastfood")?;
				alt_folder.join(file_name)

			} else if hair_name.contains("mhair_hatballcapnpc_fastfood") {
				let alt_folder = create_folder(&output_path, "mhair_hatballcapnpc_fastfood")?;
				alt_folder.join(file_name)

			} else if hair_name.contains("mhair_sideswoop_black_frosted") {
				let alt_folder = create_folder(&output_path, "mhair_sideswoop_black_frosted")?;
				alt_folder.join(file_name)

			} else if hair_name.contains("mhair_sideswoop_blond_flame") {
				let alt_folder = create_folder(&output_path, "mhair_sideswoop_blond_flame")?;
				alt_folder.join(file_name)

			} else if hair_name.contains("mhair_sideswoop_brown_flame") {
				let alt_folder = create_folder(&output_path, "mhair_sideswoop_brown_flame")?;
				alt_folder.join(file_name)

			} else {
				folder_path.join(file_name)
			};

			let resources = vec![
				DecodedResource::Gzps(hair.gzps.clone()),
				DecodedResource::Idr(hair.idr.clone()),
			];
			Dbpf::write_package_file(&resources, &file_path, false)?;
		}
	}

	Ok(())
}

fn get_hairs(input_path: &Path) -> Result<HashMap<String, Hair>, Box<dyn Error>> {
	let mut hairs = HashMap::new();
	let packages = get_skin_packages(input_path)?;
	for package in packages {
		for resource in &package.resources {
			if let DecodedResource::Gzps(gzps) = resource {
				for resource2 in &package.resources {
					if let DecodedResource::Idr(idr) = resource2 {
						let key = format!("{}_{}_{}", gzps.name, gzps.hairtone.stringify(), &gzps.family);
						if idr.id.group_id == gzps.id.group_id &&
							idr.id.instance_id == gzps.id.instance_id &&
							idr.id.resource_id == gzps.id.resource_id &&
							gzps.species == 1 &&
							gzps.parts.contains(&Part::Hair) &&
							!gzps.category.contains(&Category::Skin) {
								hairs.insert(key, Hair {
									gzps: gzps.clone(),
									idr: idr.clone()
								});
						} else if gzps.family.to_string() == "1495c099-a890-4034-9836-e9eda8a670e7" ||	// thaicrown
							gzps.family.to_string() == "272bb93f-a544-41ad-8b0e-b6324fe45e5b" ||		// fhair_hatelf_blue
							gzps.family.to_string() == "edbe8b98-4596-4781-9c47-359e2fa423e2" ||		// fhair_hatelf_green
							gzps.family.to_string() == "2c0f600b-4726-4171-934b-eb503803054d" ||		// fhair_hatelf_red
							gzps.family.to_string() == "1a0bf841-f8ad-44ea-a99b-5ffcce8825f8" ||		// mhair_hatelf_blue
							gzps.family.to_string() == "1135fd5b-b4b7-42b7-b44c-acfc27949aba" ||		// mhair_hatelf_green
							gzps.family.to_string() == "0c1d74a6-60d4-42a5-baa2-df3b723c9cdd" ||		// mhair_hatelf_red
							gzps.family.to_string() == "173aa801-b774-4376-9399-637d0dfb14d8" ||		// mrs claus
							gzps.family.to_string() == "b89bd3d2-0da0-482a-a521-7af57882cc85" ||		// santa cap
							gzps.family.to_string() == "d44a2ae5-e662-4c2f-b79a-747de4488d9c" {			// santa cap white
								hairs.insert(key, Hair {
									gzps: gzps.clone(),
									idr: Idr::new_empty(&gzps.id)
								});
						}
					}
				}
			}
		}
	}
	Ok(hairs)
}

fn lookup_group(family: &str) -> String {
	(match family {
		"53b0336c-850c-46bc-a135-d1cd58deb258" => "fhair_acorntuck",
		"f25d347b-3053-47ce-8cb8-04e19627ef9f" => "fhair_acorntuck_strawberry",
		"9ca4ea4d-0b71-443f-953c-e14e4fe31725" => "fhair_aline",
		"263420d1-6d37-4b9c-83cf-1697a6708665" => "fhair_aline",
		"04e06f98-ccc4-4357-9683-5f48882efcb3" => "fhair_bad",
		"b58105c4-f4a1-4419-a743-b6de36345076" => "fhair_bald_bare_burnt",
		"15cad1cb-e896-451d-b9d9-5179e949e62e" => "fhair_baldwitch_bad",
		"c7cbad41-f901-45f7-9015-9cd4457e899a" => "fhair_baldwitch_good",
		"588c2be9-5b65-4417-9844-acc1b3c67190" => "fhair_bandana_brown",
		"15d848a4-eb4f-4798-8438-13a6055e67ab" => "fhair_bandana_gray",
		"86663c42-2cd1-4221-81e0-882126d2720e" => "fhair_bandana_green",
		"039f10f9-7b4a-4f9a-9e30-7b7eda6f7c2c" => "fhair_bandana_purple",
		"9c5cd08f-7013-417c-854e-707fa34304e5" => "fhair_bandana_red",
		"af1b00f6-1ed1-41ab-b5e1-8b006a68aeff" => "fhair_barrett_blackclip",
		"89df0723-506f-4cdb-98a2-feb923ca8578" => "fhair_barrett_blondclip",
		"ad94bac3-6d64-4371-bb23-85520369ab94" => "fhair_barrett_brownclip",
		"f4ac9685-8171-4631-8e22-5d95abe7eedb" => "fhair_barrett_greyclip",
		"1d3b6234-b4b4-4354-89c3-a8f93844a2a4" => "fhair_barrett_redclip",
		"76a76737-63fb-40ca-ac31-ffa5d2c3cdf4" => "fhair_barrette+mob",
		"23849b4b-db3b-44c8-9083-56eacb4770a5" => "fhair_bun+bare",
		"75ed0332-cabc-4643-9469-ee63a9a826c1" => "fhair_earmuffs_black",
		"7930be3c-af8d-4e37-ba83-f3af0899d422" => "fhair_earmuffs_blue",
		"4e3c936a-bf89-4ccc-976e-87c9ec7a944d" => "fhair_earmuffs_brown",
		"09ea58c4-62a8-4a2e-a786-b298da0c976b" => "fhair_earmuffs_pink",
		"97a22f79-132f-47cf-92b7-7933a0e556f8" => "fhair_earmuffs_white",
		"7b9c2cca-7832-4361-b736-e970f50980f5" => "fhair_beanie",
		"9ff7e1af-0bf1-4b3d-87c5-dbe82393f957" => "fhair_bowltwist+formal",
		"8500e751-fb01-413c-8d34-278603db3e24" => "fhair_braids",
		"da5ffe4f-8868-4a4c-958d-64602cea5ed3" => "fhair_braidsup",
		"541f620a-507d-41bf-8f3c-a43e104934ed" => "fhair_cornrowslong",
		"660198ec-66a5-4a49-828f-6fd08405f920" => "fhair_crown",
		"40d0c022-aa29-45a1-9205-e0a25810981e" => "fhair_curlsup",
		"200c2db8-18ff-4851-b661-fd2e76ee9943" => "fhair_deco",
		"8ad4217d-dade-4199-a134-e9b3edcc8176" => "fhair_dragon_green",
		"e201099b-6cc2-424a-be73-ddcd2a13df77" => "fhair_dragon_orange",
		"373dec7e-653c-4867-a984-bcd25c4b80d5" => "fhair_dragon_pink",
		"fef7204b-fb54-4803-acfd-24b4ad235bd9" => "fhair_dragon_teal",
		"f3c2e204-2d1a-4cab-bf1a-e821574ddf49" => "fhair_dreadlockshort",
		"199e9540-a3fd-4f35-9cb3-7016e0b0950d" => "fhair_dreadsband_blue",
		"48ca6c51-41c8-4913-871e-c30b3b4c9c1b" => "fhair_dreadsband_blue",
		"c6525403-208b-49ba-85e6-05290568d450" => "fhair_dreadsband_dark",
		"a521dd84-5f94-40ec-b513-caac271b985a" => "fhair_dreadsband_dark",
		"0a9d090f-6198-4a87-aef6-459be8fbfc63" => "fhair_dreadsband_maroon",
		"23e2211f-dfc4-4225-ba7b-10cdf5371400" => "fhair_dreadsband_maroon",
		"00d442de-c903-4396-bfe4-9fc7b6915328" => "fhair_dreadsband_myrtle",
		"c5b0e65c-d37e-469b-8cd4-301714a2abd6" => "fhair_dreadsband_myrtle",
		"23a05175-e0a6-4450-af8f-4f41756baad2" => "fhair_dreadsband_tan",
		"a18f900b-83bd-49c3-a7a5-7357dea3951b" => "fhair_dreadsband_tan",
		"79310e69-9bb7-4064-be46-29951e231c9a" => "fhair_earhat_green",
		"a67d6fb4-5abc-462c-92dd-68a1880de040" => "fhair_earhat_green",
		"c399037d-9a85-4a60-91a1-a3e8d8cf888b" => "fhair_earhat_green",
		"78711d79-64d9-4ca0-b4e0-5d2a4ed31f2a" => "fhair_earhat_pink",
		"a9b4a1a5-de2b-457a-8a62-aca1144fa736" => "fhair_earhat_pink",
		"dca95c2b-9260-4a4b-b94a-1d362108a6fe" => "fhair_earhat_pink",
		"ea02b941-036b-43da-9f15-a4f695cc4052" => "fhair_earhat_purple",
		"9e3f85d0-7b55-4bdc-a413-b34f86112877" => "fhair_earhat_purple",
		"49633acf-6920-4e76-a282-2660d7e9328b" => "fhair_earhat_purple",
		"f9fbffb3-f511-4873-9fb5-ca5cb5499b96" => "fhair_earhat_silver",
		"a48c3468-24ca-447b-8b21-25086f1b4064" => "fhair_earhat_silver",
		"e659f044-cadb-4b33-b1a4-5efe5c8ecce7" => "fhair_earhat_silver",
		"46e4e445-0718-4e65-9e25-ab1bfed33384" => "fhair_earhat_teal",
		"e207eecb-bc98-4da9-8523-5d3f0088e242" => "fhair_earhat_teal",
		"a7185f9f-da52-45c0-acc4-c63b763d4cbd" => "fhair_earhat_teal",
		"90e1a6b2-8450-4a89-b2d9-a0283dce6419" => "fhair_feather",
		"81a5162d-4464-449b-98ac-02a8cf73480b" => "fhair_flamencoflower",
		"34e04525-b336-45c7-ba88-d4ce6453d7e5" => "fhair_flowers",
		"edc45966-1e01-45f4-9319-88d1aee136a0" => "fhair_flypigtails+swirl",
		"b3d610a2-e3a5-4f37-a34f-f3615450b99e" => "fhair_frenchbraid",
		"771bb139-ecf6-4f0a-9692-f320885c697f" => "fhair_fuzzylongcp",
		"823f0c51-3a25-4a60-bfd1-014eea6842a6" => "fhair_getfabulous",
		"04f8de8c-d33f-47ec-a330-daa9799893d6" => "fhair_gorillamask_albino",
		"d98fe93f-57a7-4b2d-be17-fdcc3f62c31a" => "fhair_gorillamask_black",
		"66c6fbc4-0d5d-4210-84d4-d1d9132f5225" => "fhair_gorillamask_blackbeige",
		"837d80fa-f0c0-4652-946b-d8334db1af93" => "fhair_gorillamask_pink",
		"adf6c097-13be-48b4-a6aa-9076715b9249" => "fhair_gwenupdo",
		"f8730eeb-7ff7-4dd2-a49b-3774d1051271" => "fhair_gwenupdo",
		"0acee78a-79e8-4599-8579-2e30036f18a7" => "fhair_gypsy",
		"b2c60bd4-c449-40df-8151-fe0ea0324a40" => "fhair_gypsy_ep7",
		"fbb536cd-2421-44e6-bd64-c95c27b66a9b" => "fhair_halo",
		"aab7ba8c-c200-4a5e-9ab4-fe83689fee4c" => "fhair_hatangora_dkgray",
		"14316981-0a66-4b04-9b7d-c915faa8bfc9" => "fhair_hatangora_lbrown",
		"bbc59fa1-e876-4a0a-a38f-c549a19edc3f" => "fhair_hatangora_pink",
		"aa6afbf5-1a2e-4e50-91ad-b090174cbb82" => "fhair_hatangora_spots",
		"b9a9b843-dc99-4a0c-938e-1391e97e384d" => "fhair_hatanimalcap_brown",
		"d13343eb-4415-4ff4-8fe0-372497c677f5" => "fhair_hatanimalcap_green",
		"2da22ff1-c030-4738-aba7-845ebc2cfc47" => "fhair_hatanimalcap_grey",
		"b202ed17-91a8-4c2b-92dd-762075889305" => "fhair_hataviator_black",
		"24c94d96-293b-4087-911e-4345066448da" => "fhair_hataviator_brown",
		"0d0baf94-6ec8-4dc4-8dbd-e50de39360eb" => "fhair_hataviator_red",
		"9f5d7d99-aded-4155-8606-95cc85f47e33" => "fhair_hatbadger_badger",
		"e284c917-a826-4294-a357-9cdbbcf55c7a" => "fhair_hatbadger_brown",
		"675043cf-734f-406f-86b6-233920928b16" => "fhair_hatbadger_panda",
		"82b01231-25b0-4c23-921a-9f4d73936544" => "fhair_hatbadger_polarbear",
		"d60b87fd-6ce1-4b4b-b512-4af4ac8095c6" => "fhair_hatbaker_white",
		"ab2b1c52-796c-436d-a09f-d2860b98498e" => "fhair_hatbaker_checker",
		"de9661c0-835c-4350-81aa-61e2a3dfae65" => "fhair_hatbaker_checker_ep7",
		"5a38ee8f-a105-4e2d-a7a2-9dc8c0c25079" => "fhair_hatbaker_white_ep6",
		"a143073b-c865-453e-80e3-160089530462" => "fhair_hatbaker_checker",
		"55c8e819-b039-4ac6-8590-821dfae6f522" => "fhair_hatbaker_white",
		"781ca87b-05ca-40c0-8cf0-3bb7d5ca3fcd" => "fhair_hatbakerboy_blackleatherhat",
		"6635000c-fddc-40c1-b943-87de100de9d3" => "fhair_hatbakerboy_plaidhat",
		"5c04a3fb-21d3-4241-92fa-294abd2f097b" => "fhair_hatbakerboy_tanhat",
		"36db1a09-f153-46f1-9aca-3ac82db4c6d0" => "fhair_hatbakerboy_whitehat",
		"fa654096-2be3-400f-b233-554e4eec9088" => "fhair_hatballcapnpc_deliveryperson",
		"62529680-a41a-408a-b495-855f745caead" => "fhair_hatballcapnpc_exterminator",
		"044d6b1d-6e49-44dd-a1bb-f67e228abca5" => "fhair_hatballcapup_black",
		"a8b89d1d-e22a-431a-abb8-a63a7e82f7b9" => "fhair_hatballcapnpc_maildelivery",
		"d2c088f8-393e-46c0-9f8c-86933982949c" => "fhair_hatballcapnpc_paperdelivery",
		"f9b73af1-61e0-4a3c-a77e-787fdc99fcf1" => "fhair_hatballcapup_ep7",
		"35f88b52-cc04-4ff1-9216-1eb3c16568f4" => "fhair_hatballcapup_llama",
		"9d3935c0-f1e0-4d25-997d-78a7b438f6b6" => "fhair_hatballcapup_meshblue",
		"6a86acc5-d110-4bc5-a5f9-427885d00a5a" => "fhair_hatballcapup_meshgreyblack",
		"169b32ce-686c-445e-a8e8-18b9253c0c2f" => "fhair_hatballcapup_meshredorange",
		"5a7b44cf-992e-4a61-8764-a3faa6a0b24d" => "fhair_hatballcapup_npclunchserver",
		"986c0545-78ae-481b-870b-af33f491e1a6" => "fhair_hatballcapup_pink",
		"f223d3b5-810f-4446-8fe4-25deb968084c" => "fhair_hatballcapup_ultramarine",
		"0a93392b-46ff-45ab-9bfa-c9929536079e" => "fhair_hatballcapup_white",
		"cac81a8a-b35a-4aca-bad4-6f062f4345e0" => "fhair_hatbonnet_beigeflower",
		"5e163fca-21e0-4f20-9c02-c6344702c059" => "fhair_hatbonnet_bluedots",
		"d8aef7f3-f653-443f-8482-72c399873477" => "fhair_hatbonnet_pinkplaid",
		"1f81d7fd-60f4-4f94-bd34-66d4b6c67ff7" => "fhair_hatbucket_camo",
		"c205a2f6-6911-4033-8692-462958549a42" => "fhair_hatbucket_hula",
		"a506af3d-f6e9-4c2c-ab21-13f79a9d15eb" => "fhair_hatbucket_pattern",
		"02bc5672-eff5-4be7-981f-2073e7345ddc" => "fhair_hatbucket_plain",
		"8dcf836c-c25d-4a29-9995-12c6ff456ffe" => "fhair_hatcap",
		"aec4ede6-d130-43b9-8d28-48e13fdf9eef" => "fhair_hatcapup_bluebrown",
		"cab32ef1-2690-464d-8651-1e0a3b2591c7" => "fhair_hatcapup_logoblack",
		"4786c3fc-f6a8-456d-b411-a4fa8e087642" => "fhair_hatcapup_reambrown",
		"1924713d-f403-4017-9b9c-a789c43c1132" => "fhair_hatcapup_redgrey",
		"452f3519-6ca4-449b-9ddb-91d9939f36ff" => "fhair_hatclassicstraw",
		"a67d7ed5-6d0c-4d17-b3cf-c9c398cffd69" => "fhair_hatclassicstrawup_blackribbon",
		"3f949a46-a857-466b-be79-685152932180" => "fhair_hatclassicstrawup_redribbon",
		"8d48c5aa-9400-47d4-9402-a5073f1cd670" => "fhair_hatclown",
		"9d86d325-d7ff-47b1-a5a0-380c763d8633" => "fhair_hatcowboydome",
		"d70bfaa8-fadd-4095-99ed-35aa90740be4" => "fhair_hatcowboyupdome_gardner",
		"dab09d6e-54d1-4125-99a7-dd6d33d38450" => "fhair_hatcowboyupdome_whiteband",
		"53de5dd9-4d8a-424b-add3-fab3931f1ba2" => "fhair_hatcowboyupflat",
		"fc68a680-61a5-4223-8169-7e354d0226cb" => "fhair_hatcrownprincess",
		"2cc8b8d9-e0f0-4a2f-a974-fec3dce0732d" => "fhair_hatcrumplebottom",
		"272bb93f-a544-41ad-8b0e-b6324fe45e5b" => "fhair_hatelf_blue",
		"edbe8b98-4596-4781-9c47-359e2fa423e2" => "fhair_hatelf_green",
		"2c0f600b-4726-4171-934b-eb503803054d" => "fhair_hatelf_red",
		"1ca9c3c2-431c-4400-8786-d80e596c84f3" => "fhair_hatfargo_blue",
		"79d1cab2-364e-43a7-9463-8e1b7c5ec857" => "fhair_hatfargo_brown",
		"bd0fbf4b-400b-427c-8902-9dbe1f182032" => "fhair_hatfargo_pink",
		"fe97c17a-3638-434b-a604-94b2e2f74f78" => "fhair_hatfirefighter_red",
		"0040b0a1-4eec-4bbb-a8ca-8ab8e5e87c70" => "fhair_hatfirefighter_yellow",
		"ba1aa0f5-0e6c-45df-a62d-8a5d9cefc288" => "fhair_hatfronds_browngrass",
		"f7d14a9e-9f88-4b45-8462-b7bcac336826" => "fhair_hatfronds_ep7",
		"1376fac2-7964-4ded-b42f-4acac58c3dbd" => "fhair_hatfronds_greengrass",
		"0cf07c3a-a2fc-4a6a-8bed-0eecbcffa479" => "fhair_hatgradcap",
		"e9194a08-ef58-47fe-b155-f0a9d05e3fac" => "fhair_hathardhat_blue",
		"9a9b4976-8e78-4dd5-8aa8-3115932686b2" => "fhair_hathardhat_orange",
		"c72d3c95-1bdd-4873-8cc8-ff863368e4d9" => "fhair_hathardhat_red",
		"78974e22-f4a9-4669-95f1-c9634ee4b095" => "fhair_hathardhat_white",
		"6112ee67-dfaf-452a-b14b-51f13196995f" => "fhair_hathardhat_yellow",
		"944ae06e-720a-46f1-8a2d-3ec9b7f0588b" => "fhair_hathip_black",
		"224a9ed0-a0b9-46e8-8d2e-28b547ff4c7f" => "fhair_hathip_blond",
		"1090ef42-2f5c-4911-8f07-8b2c595f1d03" => "fhair_hathip_blue",
		"043e5aef-9cd5-4b11-abac-391196a3427f" => "fhair_hathip_brown",
		"4e5bd41c-40c8-47e4-a162-c426e30e8c67" => "fhair_hathip_red",
		"e103ba99-4fd0-4a99-b78d-b4d406977376" => "fhair_hathotdog",
		"d94e6dcb-12ab-44cd-8bcd-7d03e5f741cb" => "fhair_hatllama",
		"93226da4-1ee6-4a0b-aedd-469ab5d7bf78" => "fhair_hatmagician",
		"8b89731d-e791-493a-ae11-2ad54450d3b6" => "fhair_hatmaid",
		"3b328d89-1169-4987-9d7e-42d79e6bae50" => "fhair_hatmascotdiver_black",
		"9be5aab7-7bd6-4cb3-8eae-33c7f465582d" => "fhair_hatmascotdiver_silver",
		"c5d16114-4dd9-42f7-8c89-ab9d3b8affad" => "fhair_hatmascotdiver_tan",
		"2148a6e2-7406-4f1e-bc0a-efd49111a6b0" => "fhair_hatmascotknightclose_blackplate",
		"d02ac406-1236-4483-ae8a-f0d42be09ef4" => "fhair_hatmascotknightclose_redplate",
		"c0dba44a-3599-4112-bc14-4e2308a6d947" => "fhair_hatmascotknightclose_steelplate",
		"ef57a802-caf6-49f7-abcb-db572876de23" => "fhair_hatmascotknightclose_whiteplate",
		"cbdca553-517d-4cfc-a8a9-dd90613b68b3" => "fhair_hatmascotknightopen_blackplate",
		"52368ffb-2d8c-416f-a878-390380eabf82" => "fhair_hatmascotknightopen_redplate",
		"7d5a1ff7-e8c1-4c29-97a7-3773f4f1135f" => "fhair_hatmascotknightopen_steelplate",
		"ac918d5c-7ab9-4f4f-aa83-0c9cce46641b" => "fhair_hatmascotknightopen_whiteplate",
		"b5381854-4217-465f-9160-efa43014d009" => "fhair_hatnightcap_bluestripes",
		"d5ae09fa-43a5-4349-8013-f298885e91b2" => "fhair_hatnightcap_brownpaws",
		"468d6c99-51ad-4fe5-be15-6127e487b8c2" => "fhair_hatnightcap_greenbananas",
		"98003fd1-f2e8-424d-af42-8e0b28bd93a3" => "fhair_hatnightcap_pinkdots",
		"a2de9b43-058b-4ecb-a42c-ad54565ff1d1" => "fhair_hatpirate",
		"20124448-9ccd-4066-b680-85a8ddf9fb9a" => "fhair_hatpropeller",
		"2e2df1f1-d154-405e-9185-dc05898440b9" => "fhair_hatsafari",
		"97620939-4e80-413c-b82c-b817074fc3e5" => "fhair_hatseacaptain",
		"01c34479-4423-4171-8b93-97939aad2371" => "fhair_hatsleepingmask_blue",
		"73b706e6-2dae-407a-8b00-dc52c9465ecf" => "fhair_hatsleepingmask_grey",
		"9da4f333-6c46-43ea-a151-b763e1f1dfc8" => "fhair_hatsleepingmask_purple",
		"832b1902-1574-4011-8133-45a43aa20714" => "fhair_hatsnow_blue",
		"ed0b7ba8-d339-478c-b05e-e82ad4e8d73f" => "fhair_hatsnow_green",
		"14602114-f16c-450a-9cbf-1623e9922122" => "fhair_hatsnow_brown",
		"c68b872f-bd0a-4a8f-b899-937efa5e20e7" => "fhair_hatsnow_pink",
		"5002d854-3ca7-4a04-a44b-63f50de2dab0" => "fhair_hatsnow_white",
		"22024383-f64d-480b-8d96-059f808782fa" => "fhair_hatsnowearmuffs_bluebrown",
		"3c5bdad3-ab9a-444f-be1b-6489fb803e1b" => "fhair_hatsnowearmuffs_brownblue",
		"6cb0b1f9-9d5b-4e3d-9c4b-86cf952380f8" => "fhair_hatsnowearmuffs_greenwhite",
		"5aa162b7-31d5-471f-afc3-982547956587" => "fhair_hatsnowearmuffs_pinkblack",
		"43a90832-775a-4426-876b-1db9db8a57a9" => "fhair_hatsnowearmuffs_whitepink",
		"1495c099-a890-4034-9836-e9eda8a670e7" => "fhair_hatthaicrown",
		"a0d4ca67-4655-47d1-8b57-e14016e5a250" => "fhair_hattiara",
		"e4d5c459-94da-467a-a1c3-9db80de92f90" => "fhair_hattourguide_fareast",
		"3227f507-d8d4-4001-bebf-77aa34ac1edd" => "fhair_hattourguide_mountain",
		"caecc07a-5aee-4a44-800e-392e0fcd154b" => "fhair_hattourguide_tropical",
		"a4ecdaa9-075a-4630-abbb-8948b2beb7cd" => "fhair_hattricorn_blue",
		"a6619489-c84e-48d9-8a85-3f9b7597f82b" => "fhair_hattricorn_green",
		"b4e87789-47c7-408f-b341-d55388f4eb32" => "fhair_hattricorn_red",
		"b215c6c8-627e-4024-a0d6-b9ef4eef032b" => "fhair_hatwitch_bad",
		"785432cc-80e9-4b74-babd-26dd7d8da3c4" => "fhair_hatwitch_bad",
		"c73d3a35-206e-438b-95f8-58057519e6f6" => "fhair_hatwitch_bad",
		"ae629f32-5b79-412d-b0c8-be09c872e003" => "fhair_hatwitch_bad_clone",
		"47896105-d355-4941-9812-8bc09942f09b" => "fhair_hatwitch_bad_clone",
		"a633baf1-4ce3-4880-aa14-ac45e0902aa9" => "fhair_hatwitch_good",
		"6728fbb4-326d-476f-8ef2-51679dadb15d" => "fhair_hatwitch_good",
		"4121dc7c-4a92-43fe-8530-4fde3cfbae0e" => "fhair_hatwitch_good",
		"4b8eb8a8-54bc-4b0a-b2c3-11d9816fb1ee" => "fhair_hatwitch_good_clone",
		"c7d7a3df-cd40-422f-b785-b552f84bc73f" => "fhair_hatwitch_good_clone",
		"d05324e6-a113-4522-a332-e81428309f47" => "fhair_hatwitch_neutral",
		"d214535e-d3d7-4caf-b919-3c94e7610f6d" => "fhair_hatwitch_neutral",
		"8bff7abe-982d-47ee-a253-5e0e367b8601" => "fhair_hatwitch_neutral",
		"2f85bdf3-a8bf-4fec-bb15-1872bff79c0a" => "fhair_hatwitch_neutral_clone",
		"1c1ccb30-0949-465b-b95e-f3b07d8fd286" => "fhair_hatwitch_neutral_clone",
		"af1ca87c-26b0-4d3e-8fcd-ba38df1301c4" => "fhair_hatwitchassistant_bad",
		"d5465cbb-3309-4d7b-9c40-f2f1600feeb7" => "fhair_hatwitchassistant_good",
		"167a81f2-ee39-4a2a-b20a-7dbd461a6f88" => "fhair_hatwitchassistant_neutral",
		"92e581b5-e53c-4dad-8530-09181b715a08" => "fhair_headband_aquahat",
		"6e54282a-0444-491d-b90e-502edd758a99" => "fhair_headband_blackhat",
		"89812ecf-30b6-42ed-b4b8-3fa4530babd9" => "fhair_headband_greenhat",
		"e5349566-41cf-42ee-8415-6bf89944b105" => "fhair_headband_pinkhat",
		"845d8825-e2de-4de8-baaf-71d6d35ed122" => "fhair_highponytail_blackband",
		"19183e57-2b87-4b36-92cb-cba4e13eff41" => "fhair_highponytail_blondband",
		"a753b0f9-1642-4b2c-9dc6-e8ca40ee730c" => "fhair_highponytail_brownband",
		"267279c3-22c4-4d57-a7c5-4f2b11875d84" => "fhair_highponytail_greyband",
		"a34d0760-19b7-4216-aac8-67c58531089d" => "fhair_highponytail_redband",
		"fc060700-907f-487c-b7d6-258d46dd4cee" => "fhair_lobot",
		"8b5dca8f-746f-4d01-8dec-58fbeed357b8" => "fhair_lobotvampire",
		"58406399-7371-4e03-bd1c-308985ca68aa" => "fhair_lobotwerewolf",
		"bcf00c10-07d2-4383-ba99-a64eb9b8b442" => "fhair_long",
		"57b6c7dd-8fc6-47a2-bcec-7912f39d0fb0" => "fhair_long_bella",
		"2eb3c33d-44cf-40ed-a46f-e69b7112a6ea" => "fhair_long_freaked",
		"4c4acae0-e57e-4176-bb1e-cf80330ded38" => "fhair_long_streaked",
		"6d0e6305-ee44-4abf-aef2-8ebf069bafa3" => "fhair_longbangs_ep9",
		"f03033c4-8ef3-4727-8b8c-778183b75800" => "fhair_longbangs_ep9",
		"056a5cac-7af2-4436-b012-9a65557f9a0c" => "fhair_longbangs_ep9",
		"bac729e9-b5cf-466f-8ff3-3ac12d9b4e1b" => "fhair_longpart",
		"633a0974-6729-46cc-9793-2303917f63b0" => "fhair_vampire",
		"093fa3bd-e389-4f0e-a590-aa11cd348c42" => "fhair_longsimple",
		"a99a1f1e-38c2-401a-ba7f-16291f7b23c8" => "fhair_lowbun",
		"5a56f151-5553-4383-94d2-4c84713788a8" => "fhair_maskninja_black",
		"219e99f1-ca09-41fd-8a6d-7a14ecdfbd50" => "fhair_maskninja_desert",
		"b7a53665-324b-466f-9d92-49dc1516c5a3" => "fhair_maskninja_red",
		"801d4c76-ffde-4384-9525-58be71f73ac8" => "fhair_masksuperninja_white",
		"c7567b70-06f2-46c8-a9e9-129f50c2525c" => "fhair_mediumcenterpart",
		"62a1173a-b6b2-4ad2-b18e-6eb729aff8c0" => "fhair_meg+simple",
		"ba5eda8a-dae8-4168-9093-b19f5001cc7a" => "fhair_messedup",
		"77744e07-393d-487b-85fd-e061d7588da6" => "fhair_messedup_pink",
		"8ceb5d67-f2f7-424d-90e4-7d7ce22fc0bd" => "fhair_messedup_blue",
		"c8e8c048-7499-4153-b00e-2f31242901fc" => "fhair_messy",
		"0a925646-e879-4d14-9b4c-fa160c88713a" => "fhair_messysideknot",
		"8215017e-65db-460c-9caa-90ca437aa5f6" => "fhair_midwavemature",
		"3dc607e0-d0d9-47b2-8709-b25961325261" => "fhair_mohawk",
		"8ceb4d13-254c-4f36-a4db-80ff15e35ff1" => "fhair_mohawk_green",
		"b69fedd9-7f39-4a25-b699-845517c86af8" => "fhair_mohawk_pink",
		"1305c8a2-7b2b-4c0b-9439-faa5c5816e8e" => "fhair_mohawk_platinum",
		"d140e5cd-df58-4693-9900-867934065402" => "fhair_mohawk_purple",
		"98fc2690-a980-4a3e-8846-ff10ef63549a" => "fhair_mohawk_rainbow",
		"fb0b1a65-9b5e-4fb0-94f6-17c212bf4d8f" => "fhair_mohawkspike",
		"50748582-262f-4c97-80f2-ac865a44a97c" => "fhair_mohawkspike_blue",
		"bdc37065-7cee-4ad5-b0df-8a57b41ec8b5" => "fhair_mohawkspike_cottonpink",
		"1b6a7f8b-6469-451a-85c0-4511c0f3cc57" => "fhair_mohawkspike_green",
		"33e2d93a-bd7d-47b3-bccb-755c4b0771b6" => "fhair_mohawkspike_pink",
		"173aa801-b774-4376-9399-637d0dfb14d8" => "fhair_mrsclaus",
		"1bfa1cac-5e22-483e-9304-a15a7baf1caf" => "fhair_pagepunk",
		"a3636e07-07e4-4007-b527-157e0f02ab2f" => "fhair_pagepunk_bloodred",
		"693234da-488b-421e-a7b9-ce987ce588ac" => "fhair_pagepunk_pink",
		"6ec60aa7-2676-4f79-84d4-713074c636c7" => "fhair_pagepunk_purple",
		"d9b74b82-f156-4b23-b784-6614125dea43" => "fhair_pgchoppy",
		"68c01c64-2cd6-4a30-bb21-3c8535232880" => "fhair_pgchoppy_blondstreak",
		"94e52233-d19b-4322-bd77-5b6ed655db58" => "fhair_pgchoppy_green",
		"b0a3b87c-da21-46fb-a100-46c9adddb118" => "fhair_pgchoppy_pinkonblack",
		"113d5352-f976-4577-ab18-efcf35c4ca23" => "fhair_pgchoppy_violet",
		"cdd3b25c-69a4-4ffb-9448-1a638e6734e5" => "fhair_pixie",
		"21062191-7752-48ae-a448-4fc5025721b9" => "fhair_pixie_ep7",
		"f9a1669d-a2cc-4617-85a4-5585bfe9599f" => "fhair_ponypuff",
		"df114337-c800-4d20-8714-39f31197d0b0" => "fhair_ponypuff_blue",
		"41e38939-56aa-4c7a-93fa-a18640a6afff" => "fhair_ponypuff_green",
		"58d943cc-b48f-4e92-9a4a-4dae69023bd4" => "fhair_ponypuff_purple",
		"6a0e64d6-240e-4340-b4ac-2e854346002d" => "fhair_ponypuff_red",
		"2ad209f2-6b01-4406-99ab-1f12ee26a183" => "fhair_ponypuff_yellow",
		"21012ee6-2976-4030-bb21-0bcb110a51cd" => "fhair_ponytail+pigtails",
		"f40ce047-ebf3-4b03-a254-45e3b12b217b" => "fhair_ponytailbanks",
		"6a9d56bd-4056-4896-b004-041df21303e0" => "fhair_ponytailhigh",
		"7a494287-b2b6-4cb7-a68a-2e3f2ff19a86" => "fhair_poofs",
		"ddac1081-2ee6-4244-8f43-41dca72f1116" => "fhair_punkflip",
		"3f800a5f-a1e7-4ed3-8c09-2f3b7e9b7a3e" => "fhair_ravekimono",
		"1a3c89e3-bef0-4349-ba00-283fb80bfade" => "fhair_ravekimono_black_pink",
		"703e8e82-1155-4466-9b2c-bf4704b3d0ad" => "fhair_ravekimono_brown_pink",
		"5bb8f94a-0d14-4b16-808f-77d3f35b0577" => "fhair_rockabilly",
		"1b902cd8-ed69-4896-ba9e-b0f8215588a7" => "fhair_rosettes+puff",
		"bcec51f7-ce59-4260-b661-479a86aff468" => "fhair_shortcute",
		"8b574c1a-130a-4625-941f-0d3b5063b276" => "fhair_shortcute_frostyblond",
		"f93ea9bc-b737-4f1e-b5dd-34406be3a88f" => "fhair_shortcute_frostypurple",
		"f1502261-6ce7-4aee-9309-a1dfd919eae6" => "fhair_shortheadband_brown",
		"1ddbcd89-4e9e-4bc9-a21e-c4608c33d945" => "fhair_shortheadband_green",
		"8f182698-13de-4be9-9ba8-1567b397f955" => "fhair_shortheadband_pink",
		"caf9019f-34ad-4942-abb4-0d582beba914" => "fhair_shortheadband_purple",
		"91b40d8c-7362-4b80-973b-99cd86ad8862" => "fhair_shortheadband_teal",
		"f2114c7e-832f-454b-b03d-22e3169d6aea" => "fhair_shortperm",
		"726c8c00-0ce1-4aed-a2c9-1d616c18cabd" => "fhair_shortslick",
		"6e9f53fe-da20-4362-9918-ba0cec22d572" => "fhair_shortslick_clone_clone_HIDDEN",
		"8a715faf-84d6-422c-9540-1ba20152e936" => "fhair_shorttuckin",
		"bf5eb0d4-64ec-474b-8f99-45470395d947" => "fhair_sweep",
		"76c7258b-0b98-4cb3-943d-7a5692a5d667" => "fhair_sweep_ep7",
		"1b52b983-da4f-4ca7-8132-a10fe61257e7" => "fhair_towelturban_blue",
		"af215934-196f-45be-a9c7-d64618575b1a" => "fhair_towelturban_brown",
		"48ab0813-e1e4-4713-ae4d-b61a21acf167" => "fhair_towelturban_pink",
		"b276a951-8345-4d9b-aa44-12b6d6844052" => "fhair_towelturban_white",
		"e9be725a-2a53-484d-8a48-e359787f7e1e" => "fhair_updoo",
		"b843b9ee-7ad8-49bf-966a-2e4ba3ba4e9a" => "fhair_updoweddingveil",
		"c30d5c96-db3b-4c8f-bb97-0c02e01f78b3" => "fhair_veilcurlsup",
		"81299f25-4711-4aba-b111-4b00cca9a27f" => "fhair_weddingtiara",
		"93f70fd6-b9e7-4058-8484-761f2555eea3" => "fhair_wreath",
		"7400178f-e511-4a9e-a3b0-a9ad68cd90d1" => "tfhair_bun+pf_bald_bare_HIDDEN_7400178f",
		"530792a6-6090-4089-9f8d-335daa6e6aa0" => "tfhair_bun+pf_bald_bare_HIDDEN_530792a6",
		"d53f8765-f0ea-403f-9b69-10712c5a2658" => "mhair_achilles",
		"87da0320-6d1f-4e15-a07c-780d2682b7db" => "mhair_achilles",
		"21b4e4e9-7b1b-43ee-abf1-eb0055051887" => "mhair_achilles",
		"754df541-ab29-475d-a3f1-e9dbf8852d36" => "mhair_bad",
		"b58105c4-f4a1-4419-a743-b6de36345078" => "mhair_bald_bare_burnt",
		"6136184b-4812-4471-80b0-06ab6e5b329e" => "mhair_bald_bare_shaman",
		"b58105c4-f4a1-4419-a743-b6de36345077" => "mhair_bald_bare_tan",
		"157a67b5-37e5-493d-9b37-ae918a8e32c4" => "mhair_baldancient",
		"eabc6671-5ffa-4d29-ad10-8741e992cbff" => "mhair_baldgenie_purple",
		"07fbc23e-b288-491b-87da-1b58bc796130" => "mhair_baldwitch_bad",
		"c4b2e238-256b-484f-b22a-0f0f89a43c94" => "mhair_baldwitch_good",
		"50d2b3c6-5e2d-48ff-80e3-e7f678e72ae6" => "mhair_bandana",
		"f64024d6-eeb2-413f-ae17-1aae6647c519" => "mhair_beanie",
		"d432dcf5-786b-43b3-be2e-752f2ef0a068" => "mhair_bigfoot_brown",
		"c76b852f-40b9-4cac-a072-80452f5066b9" => "mhair_bigfoot_white",
		"12f6d2ef-953b-4eb3-9b8d-ea62f021e910" => "mhair_bigfootvest",
		"6a743afc-0027-43ec-99ab-676efff070b8" => "mhair_bigfootvest_white",
		"5d301e9c-7b93-41e4-aa8f-2395ac359724" => "mhair_caesar",
		"e4edf402-05a4-46d4-ac92-2e8b45cf310b" => "mhair_choppypeak",
		"48b77f24-5b4e-4243-b548-ee106f6dd053" => "mhair_closecrop+puff",
		"528d4155-b9db-4abf-bddd-1d0c8624a9c4" => "mhair_combover",
		"51268afd-16d6-4b3b-a4dc-25cb20d8d18e" => "mhair_cornrows",
		"f2518867-5e8d-4d26-b006-24a86f528b99" => "mhair_crewcut",
		"17cd3228-b96c-4dfe-9ef3-216fbf938587" => "mhair_david",
		"17d8d7a7-8fba-4567-94b6-362a33aae06a" => "mhair_david",
		"aa3c2473-5652-4bb3-a827-279d7efd6844" => "mhair_david",
		"d4f68adc-808a-48fa-9400-2090cdf44fb1" => "mhair_dragon_green",
		"8a2d9341-809b-4e3c-ae6e-022f59a42d61" => "mhair_dragon_orange",
		"f7a735cc-bd9b-4f57-9f36-474fc442c9e3" => "mhair_dragon_pink",
		"76c8eb8f-546f-4e8e-a92b-6fa0d01d337f" => "mhair_dragon_teal",
		"94a94829-e5e5-4245-82d9-e002fc78eb66" => "mhair_dreadlocklong",
		"cd49bf6a-c241-4617-a3b5-35a62fe3c5b9" => "mhair_dreadlockshort",
		"22b7db2e-ad7a-4c3c-bf44-5e52f3efaeb3" => "mhair_dreadsband_blue",
		"ef08cac1-6f8a-4609-a7ad-5e6cfc66d2cf" => "mhair_dreadsband_blue",
		"21653f88-5bb8-4241-8698-d3fce308d4fc" => "mhair_dreadsband_dark",
		"62c165f7-0c5d-409d-8d9e-378908226a77" => "mhair_dreadsband_dark",
		"54f7a6d3-4863-42ba-be96-ac166b75c4cc" => "mhair_dreadsband_maroon",
		"fa8d0e69-2b82-44e2-a5b8-568e8f31eb75" => "mhair_dreadsband_maroon",
		"b94c7eca-222d-4882-a0bb-1f6e593ed016" => "mhair_dreadsband_myrtle",
		"61655e52-2132-4084-8804-7aab3579974e" => "mhair_dreadsband_myrtle",
		"f37fac51-39e4-447f-a239-842614cffbab" => "mhair_dreadsband_tan",
		"cba96e77-739f-4e5b-a68d-37de5680ec5d" => "mhair_dreadsband_tan",
		"53525264-f43f-4adf-b0c0-cf27fcc2180e" => "mhair_earmuffs_black",
		"983cc5dd-7af9-47cd-8cc5-9cd1b30d388c" => "mhair_earmuffs_blue",
		"0fd31fd4-5671-4518-b21f-4c89d4254a76" => "mhair_earmuffs_brown",
		"1cbdc38e-5f75-4d69-815d-e430deea92d3" => "mhair_earmuffs_white",
		"69312b6a-b50b-4a08-bdcb-4e27af802942" => "mhair_fauxhawk",
		"ca6ab1b8-69f7-4395-bf4a-837a8c74d560" => "mhair_fauxhawk_blondtipped",
		"dc30ff3e-f674-407d-9bfe-b2eb39c09625" => "mhair_fauxhawk_green",
		"b94bbfce-0425-4211-9e1d-a5142d187b5d" => "mhair_fauxhawk_redtipped",
		"6bba0606-eb0d-4fa9-9f31-beaafc78b928" => "mhair_frontalwave",
		"a1e0fd9e-0e56-4dbc-895e-70ff05d14a23" => "mhair_gelledrock",
		"eba33d19-a86b-4e4b-a458-74942ec6dc7e" => "mhair_gibs",
		"23992192-15f9-4593-b1a0-aac44eeecd3b" => "mhair_gorillamask_albino",
		"e20885e6-4366-4b5a-98ff-2b94ef9dd74e" => "mhair_gorillamask_black",
		"dd1bfb76-27d4-4a8c-804f-a3a3f68f6c32" => "mhair_gorillamask_blackbeige",
		"b67036c4-574f-4152-b3f0-4ac0fac17312" => "mhair_gorillamask_pink",
		"ebf0636d-6449-4d6d-84f5-49ce06bd5449" => "mhair_hatanimalcap_brown",
		"8e59e410-762d-4090-abdc-5856f3929f97" => "mhair_hatanimalcap_green",
		"d2fc9d4d-28a1-460c-900b-52b59fa1569d" => "mhair_hatanimalcap_grey",
		"2c17e8e4-1d83-46ae-8f73-651ad776affd" => "mhair_hataviator_black",
		"6c6e4e49-d9b8-43bf-bdaf-0bc80003a934" => "mhair_hataviator_brown",
		"a657912a-4af6-478b-8573-9ebe90da1a1b" => "mhair_hataviator_red",
		"0b7b9c9c-4559-47a9-bf4c-5572eee489fb" => "mhair_hatbadger_badger",
		"1e4bbf89-b7df-42dd-ab89-166f6d7d96b8" => "mhair_hatbadger_brown",
		"3b97367d-4411-4e7e-8c89-115d2f0b01c2" => "mhair_hatbadger_panda",
		"24802d2c-8c52-4320-9e9f-e84dfb2fd124" => "mhair_hatbadger_polarbear",
		"f294e3c8-e40f-4db8-83ff-eee8e1524ed8" => "mhair_hatbaker_white",
		"ae1d17cb-026b-4121-8ac7-759c14adeea3" => "mhair_hatbaker_checker",
		"3b93f1a9-ed9c-42f5-ba21-b1ae68bf5952" => "mhair_hatbaker_checker_ep7",
		"efee9ea6-07e2-4216-b03b-18f22c54e272" => "mhair_hatbaker_white_ep6",
		"b4f40b73-6ce4-4e54-bc96-3bf8c3bd45ba" => "mhair_hatbaker_checker",
		"1ef02c51-80bd-40b8-b89d-7cf50417ac82" => "mhair_hatbaker_white",
		"e9436d60-e5f4-4403-a7d2-bc655b576d43" => "mhair_hatballcap",
		"972a64b1-ee1c-45c2-b26b-fe2fbab7115a" => "mhair_hatballcapnpc_deliveryperson",
		"43ac6fd5-c38f-443a-a69e-412d69a8548a" => "mhair_hatballcapnpc_exterminator",
		"47d64200-63eb-4c1a-a91f-6ad836008f28" => "mhair_hatballcapup_black",
		"0ecd3f21-fe17-4760-a1a7-adb79ba5cf7c" => "mhair_hatballcapnpc_maildelivery",
		"999eeeda-2def-445d-afeb-dddbe002073a" => "mhair_hatballcapnpc_paperdelivery",
		"6519ecce-23d3-4346-b2d8-7735c508ddbf" => "mhair_hatballcapup_ep7",
		"3674aecd-31a2-4a5e-8870-e67de693aa9a" => "mhair_hatballcapup_llama",
		"cc38be16-33b3-494c-abd2-1b8142b13cdd" => "mhair_hatballcapup_meshblue",
		"1d5f4c37-16c8-4ba1-ae31-5d85196bbf15" => "mhair_hatballcapup_meshgreyblack",
		"6f2e8f52-93df-489e-826d-c895b22ccff5" => "mhair_hatballcapup_meshredorange",
		"581dfafb-7922-4b7f-9b9d-11896fd3dd1f" => "mhair_hatballcapup_npclunchserver",
		"1b33d608-42fd-4d42-a3cc-0c200a4bec30" => "mhair_hatballcapup_pink",
		"7d9a646a-e900-418c-81dc-eb68b3461947" => "mhair_hatballcapup_ultramarine",
		"4eece87a-4060-44b1-b2c9-d423cb7f5cd9" => "mhair_hatballcapup_white",
		"6b7b7435-060e-48de-a302-25dfa07bcc4d" => "mhair_hatbeardgenie_beard",
		"5c59a504-c6a2-41a0-be8f-be98115e4411" => "mhair_hatbellhop_grayhat",
		"4eee7ee0-e75a-4d78-ae4e-14b53f8d018d" => "mhair_hatbellhop_navyhat",
		"e4c145dd-4ba5-4b5a-874f-4961da2d5ec3" => "mhair_hatbellhop_redhat",
		"7cfd0e1e-8a22-405f-b0c2-d42e22c7456e" => "mhair_hatbellhop_taupehat",
		"a524a792-b1bb-441b-a8c5-e98b817ccbca" => "mhair_hatberet_camo",
		"7e682d56-607b-447c-9b9e-9df873856b88" => "mhair_hatberet_dkblue",
		"4f2060d1-6a9d-4dad-b4a6-ece888aa3009" => "mhair_hatberet_green",
		"181c3c12-f64f-4bf5-b1c6-42d99f4babe5" => "mhair_hatbigfootwitch_bad",
		"e192de97-1cfa-4670-a550-4198e2edb934" => "mhair_hatbigfootwitch_good",
		"d2bfda2e-08e9-48f2-9cd5-3bf8f8df36ab" => "mhair_hatbigfootwitch_neutral",
		"1c668565-43d8-4618-8cf1-cc190a5bd844" => "mhair_hatbucket_camo",
		"2894c881-7b16-4f8c-a1e4-2cecb4f3f9f9" => "mhair_hatbucket_hula",
		"d5f5735f-6ce9-4755-a0ee-b1688ef3ad35" => "mhair_hatbucket_pattern",
		"4b9cd74f-30ac-4594-a406-7cc1a7660ec4" => "mhair_hatbucket_plain",
		"14e1e322-b116-47a6-a525-45ef82db0709" => "mhair_hatcap",
		"ab9710d0-2b0f-40e2-8faf-b1a616889630" => "mhair_hatcapup_bluebrown",
		"8970ab16-6eeb-4c71-8bb2-21dcbb72b904" => "mhair_hatcapup_logoblack",
		"41666a06-db27-4d5a-97d5-e747ac1b839a" => "mhair_hatcapup_reambrown",
		"77d2a0f6-b2fd-4f93-a5ea-de78cbe64036" => "mhair_hatcapup_redgrey",
		"5430879d-4d07-45c3-b000-df9d5890ca30" => "mhair_hatclown",
		"f7962a5c-d360-472a-a2d4-214a03623ee7" => "mhair_hatcowboydome_gardner",
		"4504b710-926b-4883-8607-8e5ce984190a" => "mhair_hatcowboydome_whiteband",
		"30750321-52de-402f-bf06-7f41bf1fa65a" => "mhair_hatcowboyflat",
		"1a0bf841-f8ad-44ea-a99b-5ffcce8825f8" => "mhair_hatelf_blue",
		"1135fd5b-b4b7-42b7-b44c-acfc27949aba" => "mhair_hatelf_green",
		"0c1d74a6-60d4-42a5-baa2-df3b723c9cdd" => "mhair_hatelf_red",
		"9e9c10fc-6c3c-4297-a242-e9ea0c22f831" => "mhair_hatfargo_blue",
		"7fed6fb3-22b5-480f-83cd-236527b01780" => "mhair_hatfargo_brown",
		"7dae1ee7-2764-4b86-9e50-8d85b9ab35d0" => "mhair_hatfargo_pink",
		"12bf7727-7eb3-4dec-b989-27064428adfe" => "mhair_hatfedora",
		"b5e54d4e-5676-45a2-a241-a661eb7dc844" => "mhair_hatfedoraband_black",
		"53dfbeac-654a-4fb7-836c-48ccadde7e24" => "mhair_hatfedoraband_blond",
		"e719ad59-267d-44f1-8ec3-f8f2db30b90b" => "mhair_hatfedoraband_brown",
		"e844fc12-07a8-4f10-b414-86608830ee16" => "mhair_hatfedoraband_grey",
		"343d78ff-9c24-4b06-8c8e-1cd25617f1c6" => "mhair_hatfedoraband_red",
		"66f5eb08-81dd-45b2-8b44-8ef1c17ed839" => "mhair_hatfedoracasual",
		"3dae6d1f-3c1a-443e-aabf-d127bcfaf7ee" => "mhair_hatfirefighter_red",
		"43fc29ea-8511-48f4-a9af-028c6bbb302a" => "mhair_hatfirefighter_yellow",
		"c963e1c0-5397-4ec8-b0bc-7368a5894eb4" => "mhair_hatflamencohat",
		"d9efa60f-b7a9-437b-968f-2ed7752f9ecc" => "mhair_hatfronds_browngrass",
		"0185b26d-a290-4678-b9fc-4383fad52d36" => "mhair_hatfronds_ep7",
		"6c9346f2-953d-4ce6-a8e0-2ee232d5e0e2" => "mhair_hatfronds_greengrass",
		"0096bb7c-675d-46ac-bc4e-eddbc56b15e9" => "mhair_hatgenie",
		"0cfd8027-6a6f-428f-aa2d-c3689cabfaaa" => "mhair_hatgradcap",
		"ef94a18e-84ef-498e-be51-fdd6bee48c86" => "mhair_hathardhat_blue",
		"e020bcf2-e3a7-4bd0-898e-bc452aee8e17" => "mhair_hathardhat_orange",
		"77b6c31f-69b3-4d2f-b468-f879db393e46" => "mhair_hathardhat_red",
		"4398f09e-4df1-475e-ad10-a8cf0f520cea" => "mhair_hathardhat_white",
		"a2b4c0b2-c68c-4cbf-93d1-f64b552fb8d0" => "mhair_hathardhat_yellow",
		"311d4dd4-d36e-4a38-842c-11675baa6f1f" => "mhair_hathotdog",
		"f5217aea-044c-49c0-a001-4748dac3ee1b" => "mhair_hathumanstatue",
		"4a6e84e1-a914-4998-bd5c-2afbaf3c730d" => "mhair_hatkilt_artsy",
		"9a349655-5f74-4396-b51b-d54dbb78ac5f" => "mhair_hatkilt_leather",
		"fe8021f4-ab20-41a9-a726-1639d2d9d9bd" => "mhair_hatkilt_scott",
		"b940d0fe-4f8c-46fb-8471-b3f86318fa07" => "mhair_hatllama",
		"e06f21ab-7db3-4682-8c8a-f0dffaa19ff5" => "mhair_hatmagician",
		"57c84741-a4d8-430f-acd4-82f6348cd147" => "mhair_hatmascotdiver_black",
		"9550c73c-3bae-4699-aee3-8b44e0a83616" => "mhair_hatmascotdiver_silver",
		"40a8a2d9-2603-4231-85cc-35dfa9b66c20" => "mhair_hatmascotdiver_tan",
		"3b22e32a-a0a0-4063-b2a6-404b5717e45f" => "mhair_hatmascotknightclose_blackplate",
		"ea811eec-eb9c-4d8f-abb5-407737bf36a7" => "mhair_hatmascotknightclose_redplate",
		"4a0722cc-cc9a-49c3-a1d0-c332f865bed9" => "mhair_hatmascotknightclose_steelplate",
		"85726d14-4f12-4a9d-99b2-c61f28b73be7" => "mhair_hatmascotknightclose_whiteplate",
		"826f8746-3e2f-4174-a062-269efe4483ee" => "mhair_hatmascotknightopen_blackplate",
		"bac40a3b-64a5-4c07-ab70-26385fc048fe" => "mhair_hatmascotknightopen_redplate",
		"c53bae60-b05b-4ecb-8c4a-3301aa598bcd" => "mhair_hatmascotknightopen_steelplate",
		"e00b597b-9a7f-4025-adc6-8c091c7a224a" => "mhair_hatmascotknightopen_whiteplate",
		"f9b384b3-63d6-40ae-a86c-ef4e2bff9613" => "mhair_hatnightcap_bluestripes",
		"28fce77d-bc18-4969-90db-ac48f72c01ff" => "mhair_hatnightcap_brownpaws",
		"9fe26cf9-405f-4ea0-b2b3-929b58de81de" => "mhair_hatnightcap_greenbananas",
		"dfb51137-f0e5-434b-a4bf-4baf80c731bc" => "mhair_hatnightcap_pinkdots",
		"4793b50b-c7cb-4433-9f9c-662b9ebb568b" => "mhair_hatpanama_browntan",
		"6591001b-48f4-47ce-be4b-37c02ec53be2" => "mhair_hatpanama_stripes",
		"61869442-a6aa-465f-b5e2-4a1a9a8cbdd7" => "mhair_hatpanama_thinband",
		"2ce55e66-7491-4c5a-af56-7d85b39e9e13" => "mhair_hatpaperboycap_blackcap",
		"fc732d95-e661-453a-a34f-c1838dac0827" => "mhair_hatpaperboycap_blondcap",
		"7970b8c3-887f-4705-8e9f-6b6ce0b16fca" => "mhair_hatpaperboycap_browncap",
		"b4807346-9a80-49e9-b8a5-cbe7833bd067" => "mhair_hatpaperboycap_graycap",
		"920217ed-5ec1-4683-b489-4b8be79b30e5" => "mhair_hatpaperboycap_redcap",
		"58207b21-0e39-460d-92c5-bf0fad621382" => "mhair_hatpirate",
		"80a3fa7e-d007-43a7-a782-0f51a9b2f883" => "mhair_hatpirate_ep6",
		"d32e7fad-acc3-427c-9a1b-238a6d4ee463" => "mhair_hatpirateeyepatch",
		"0130f5d2-f132-45b1-af35-37d21228ff75" => "mhair_hatpropeller",
		"c3ac48a9-d016-42c7-977f-a091119520eb" => "mhair_hatsafari",
		"1d8c883b-34f4-4615-b96f-fe017cfc2025" => "mhair_hatseacaptain",
		"57e642e9-ee89-4dac-bd87-dc7c0dd1fa9e" => "mhair_hatsnow_darkbrown",
		"6f6e8243-bab4-4f0d-8369-82a9d8ff59ca" => "mhair_hatsnow_darkred",
		"c0d29737-a8c8-494f-ba0b-18f05d319650" => "mhair_hatsnow_green",
		"0e17da43-7828-4447-82a5-cc8700a4d641" => "mhair_hatsnow_navy",
		"82bb5bd1-db74-4c2c-b579-41ab9c626156" => "mhair_hatsnow_white",
		"c8bf8845-a11a-4765-a961-3a8832201e5a" => "mhair_hatsnowearmuffs_brownblue",
		"b5e78b57-6633-4806-a224-d0a8a579b8b2" => "mhair_hatsnowearmuffs_greenblack",
		"a64ad31c-9036-4f8c-895a-e367ff6ac40a" => "mhair_hatsnowearmuffs_navywhite",
		"a26ce229-f998-41c4-b30b-35cddb850f08" => "mhair_hatsnowearmuffs_redwhite",
		"a89be755-1b01-4e8c-a3e3-2298c7d88b4f" => "mhair_hatsnowearmuffs_whiteblack",
		"a17e7302-6a2f-4c8a-a7af-7fd4de8c0889" => "mhair_hattourguide_fareast",
		"49e80971-9a5b-44e5-9cf4-45af1392a384" => "mhair_hattourguide_mountain",
		"a7aa909e-abd2-4920-b85a-2420a960917f" => "mhair_hattourguide_tropical",
		"d2abbe7e-9f7a-49b1-91f5-ede3811b7484" => "mhair_hattricorn_blue",
		"2da14822-6bea-4814-bc57-184068ed476b" => "mhair_hattricorn_green",
		"5bea7f82-0c86-4b13-b962-8722a08583ec" => "mhair_hattricorn_red",
		"98ee3544-35c1-4d6c-a531-bad73b01e683" => "mhair_hattrucker_bluehat",
		"9a238b10-a587-4fe6-9152-62531beff6b3" => "mhair_hattrucker_brownhat",
		"5ab44c9a-43b9-4438-8f5e-a8523d699359" => "mhair_hattrucker_greenhat",
		"ce5574cc-d80b-4f7b-b1da-6b4a9cffe7f3" => "mhair_hattrucker_greyhat",
		"30f5fac3-e25f-469c-99d3-e3563f6a8a24" => "mhair_hattrucker_redhat",
		"7bfe1613-a1ca-41cb-b651-6bacf6bd64b6" => "mhair_hatviking_black",
		"0a7dafd7-f072-4e14-a369-2d2fe69a31b4" => "mhair_hatviking_white",
		"5a0b74d4-5a86-494b-9b7c-90b1b857fc64" => "mhair_hatvillain",
		"879a1009-17b5-480e-9abf-483d94194220" => "mhair_hatvisor_brownhat",
		"c20175cd-d3fe-4792-acc9-19c59d1792eb" => "mhair_hatvisor_grayhat",
		"3ead60f5-c366-4965-ac35-30af91918a77" => "mhair_hatvisor_greenhat",
		"2d5bcdad-4601-43a9-af3b-ad4801dcf896" => "mhair_hatwitch_bad",
		"0e2f67fb-1f18-41b9-a397-8b8f59982f67" => "mhair_hatwitch_bad",
		"4dafc4a2-1815-434a-9f8f-d2994ca70abb" => "mhair_hatwitch_bad",
		"dd479b71-7728-4b70-9dc1-6e7b4def63ec" => "mhair_hatwitch_bad_clone",
		"661742e9-d911-4b1a-ba40-ece04694fd15" => "mhair_hatwitch_bad_clone",
		"5272a423-e277-4b8e-a83c-8f26b78c0c60" => "mhair_hatwitch_good",
		"7f11f0cb-5622-474f-9f6e-e4e192cb7f59" => "mhair_hatwitch_good",
		"91bfce9c-565a-4f6b-b1f7-18d5fcce3fd5" => "mhair_hatwitch_good",
		"b2fd5329-4643-43b5-b8b0-5964748cb6b5" => "mhair_hatwitch_good_clone",
		"850b7962-f643-4142-bb8d-1c5c896effe1" => "mhair_hatwitch_good_clone",
		"a31a1306-646c-4ce0-bca6-1e28baa4ed17" => "mhair_hatwitch_neutral",
		"fd4a8e41-21f6-4e2a-9c3f-eefefe1df28f" => "mhair_hatwitch_neutral",
		"252ec6b2-090a-4703-ab89-2f8cbf55a2fa" => "mhair_hatwitch_neutral",
		"de5d5aa1-547b-44ae-937e-a622e286531d" => "mhair_hatwitch_neutral_clone",
		"93fc9309-12f6-4158-82c2-1c59eeda6fad" => "mhair_hatwitch_neutral_clone",
		"89785c18-f0b3-4438-b8db-dc0968f32967" => "mhair_hatwitchassistant_hat_bad",
		"c54a8a30-13c0-450b-983e-c9e8ab9d2386" => "mhair_hatwitchassistant_hat_good",
		"05beb5f5-b4d7-4030-a6f8-6eb3bfe551f8" => "mhair_hatwitchassistant_hat_neutral",
		"94a8f81f-325a-4fdc-afa9-351fad86f52f" => "mhair_lobotvampire",
		"cbb671cf-2569-43a6-8d91-d0574f16c1cf" => "mhair_lobotwerewolf",
		"bfb8e5d3-5c77-4b34-bfcb-c47412e73a88" => "mhair_longbangs",
		"c73cd61e-c499-4b8e-ba07-a40020f9d6c9" => "mhair_longbangs_ep7",
		"9a5dcaa8-b2fe-45a1-92cc-d02a1b27dbff" => "mhair_longsimple",
		"3b4ea874-2be1-4a70-8edf-b5a61fc00111" => "mhair_longsimple_ep7",
		"84737935-771c-4ea0-ad21-a3ca4913879a" => "mhair_maskninja_black",
		"94330db1-f9c2-4cd6-8283-4571c780bac8" => "mhair_maskninja_desert",
		"e8e01cf7-5d43-43ff-9de5-1df9ab7dcdbd" => "mhair_maskninja_red",
		"fefaf88d-8fdb-4359-90bf-fcd1ebaf713c" => "mhair_masksuperninja",
		"d20579c7-ad52-4a48-a87d-f5d9e8e13164" => "mhair_messy",
		"724e15b9-742a-4cd6-a687-d3f8febed69e" => "mhair_messy_clone",
		"d306d927-5f79-43b1-a5c8-de87615286b8" => "mhair_modbangs",
		"1d2a0199-e86c-48d0-9c30-e7d31464922a" => "mhair_mohawk",
		"f379469d-fa9d-41f8-9cc3-270cbb17e41d" => "mhair_mohawk_flame",
		"92a4fa15-1f4a-4078-9b1f-01b674c3aa9b" => "mhair_mohawk_green",
		"870b08fc-f054-4567-9b41-efeadb5e5d7e" => "mhair_mohawk_pink",
		"7784008b-28fd-4f3a-a85a-5cfecd1c6c45" => "mhair_mohawk_platinum",
		"37884bb9-e3bc-4b20-b917-505930b1c63e" => "mhair_mohawk_purple",
		"ef2fb65a-0b66-4988-a17c-5e2849575557" => "mhair_mohawk_rainbow",
		"544de81d-b32e-47ff-a1b3-92d8737ca5e5" => "mhair_mulletlong",
		"78e9e609-efc5-4ae4-8330-e0d7e9c0a6aa" => "mhair_mulletlong_blendblond",
		"38103f06-8359-47be-84df-753678f6f910" => "mhair_peak+ricebowl",
		"f226f4e1-18d2-4356-aba8-0095cb5b8694" => "mhair_pgskater",
		"9fbc32a6-220f-4f14-bde9-2e1b2776f3ca" => "mhair_pgskater_blend",
		"910b923f-0aa4-4e08-8ad9-ef85a4d9480c" => "mhair_pompodore",
		"1b3fac2d-152c-4eae-b307-da545751e5c3" => "mhair_ponybandana_blue",
		"92977668-0a37-49d6-a854-f5508134ccef" => "mhair_ponybandana_green",
		"c392c022-4026-4b61-bce9-90fee4a85fef" => "mhair_ponybandana_red",
		"0247ab23-0f5f-42d0-ba09-2a75a35c50e2" => "mhair_ponybandana_white",
		"2aafc175-1628-4a53-bf3f-e8994f2d73ed" => "mhair_ponybandana_yellow",
		"b7778fe4-fdb8-4c7c-9988-566dd6e26237" => "mhair_ponytail",
		"08df2ffb-8d78-4247-8590-d67e6e36c75e" => "mhair_ponytaillong",
		"fcef2a15-948b-4540-a91b-957a85f5b373" => "mhair_ponytaillong_ep7",
		"f1b5daf7-09be-48a5-9dc1-4104c6a37d53" => "mhair_realcloud",
		"a31a5ea6-fe06-4088-8d0f-024ac4514ced" => "mhair_realcloud",
		"a94bff77-5a18-47a9-b213-9bec1bcc2669" => "mhair_realcloud",
		"dde0bfd9-1814-4921-a400-26cdaa3d08c4" => "mhair_rocker+simple",
		"cba415ec-92b4-4c6c-8edf-1b5a2d837ead" => "mhair_rocker_blend",
		"b6e67c8b-ade3-40cb-ba41-69d372f44b86" => "mhair_rodhumble",
		"6453e9a8-61e0-40b9-9da0-29e36e885678" => "mhair_round",
		"b89bd3d2-0da0-482a-a521-7af57882cc85" => "mhair_santacap",
		"d44a2ae5-e662-4c2f-b79a-747de4488d9c" => "mhair_santacap_white",
		"12617bc5-e7c5-4578-afae-185375cd5f1b" => "mhair_semibald",
		"e1e7f578-b777-456e-b9dc-5c76ec9ba3c2" => "mhair_short+swirl",
		"8eb49f3c-3c4a-484d-af9f-474523aff1aa" => "mhair_shortcombed+bare",
		"ef4a4647-c327-4b75-b0a7-852e5492be6c" => "mhair_shortcombed_HIDDEN_ef4a4647",
		"13d88528-ecfe-456b-a32f-1015be7ebf26" => "mhair_shortcombed_HIDDEN_13d88528",
		"97c76133-155e-4c7c-955f-bc1c928938d5" => "mhair_shortcombed_HIDDEN_97c76133",
		"1a992f31-b456-43ff-b789-d8b11159d889" => "mhair_shortgel+stubble",
		"bef929b6-01ba-4171-86bc-213347b54c44" => "mhair_shortgel_blue",
		"53f42d90-06b3-4ba7-9f41-14a9bbc7acc3" => "mhair_shortgel_orange",
		"409a6c30-3c99-4803-ae26-35f138a0a52e" => "mhair_shortgel_purple",
		"462bbad1-1802-481c-af7a-08c26c584717" => "mhair_shortgel_HIDDEN_462bbad1",
		"b1b443b8-8854-49d7-8a1d-fa2e15b30766" => "mhair_shortgel_HIDDEN_b1b443b8",
		"e59c4200-f263-44d7-abe5-769dc18f2b56" => "mhair_shortgel_HIDDEN_e59c4200",
		"b489e696-a93d-46b0-b878-0c1a0c05940c" => "mhair_shortmop",
		"fce9883f-7c85-4a27-b2cd-d5a92f0a7066" => "mhair_shortcenterspike",
		"a66ae419-d8ae-4bb0-bee0-280a467a1cf8" => "mhair_shortsimple",
		"6b6fd581-77b6-4cc9-9d95-25320912f416" => "mhair_shrink",
		"051f1bb7-0337-4340-a45a-d4ed453a7e2b" => "mhair_shortspikey",
		"7358f5dd-2dca-49b2-9ce3-baf5904d5f04" => "mhair_sideswoop",
		"0c3f8ab9-ce31-4654-aff7-a90c0b3909a1" => "mhair_sixty",
		"e38a1cd6-51bb-46f5-b874-a7f4fc334a31" => "mhair_surferteen",
		"527e8430-586e-4f12-9edb-e4472711a55e" => "mhair_tophat",
		"b3531c0f-e4d5-4fb7-96b4-02a488ae2826" => "mhair_will",
		"3481288b-a91e-4dac-a0a3-c662497ee024" => "pmhair_bald_swirl_HIDDEN_3481288b",
		"321695f8-6c3c-481e-90c0-045c3a0a42d9" => "pmhair_bald_swirl_HIDDEN_321695f8",
		"5a1f8b61-c38c-405e-9f63-d14c9ae805ae" => "pmhair_bald_swirl_HIDDEN_5a1f8b61",
		"53752ea9-d740-47d9-a693-47fcf7fda87e" => "pmhair_bald_swirl_HIDDEN_53752ea9",
		"b58105c4-f4a1-4419-a743-b6de36345079" => "uhair_bald_bare",
		"956b950a-e0e7-4d91-9d06-ec2f7345f6e7" => "uhair_bald_frost",
		"382ed359-8529-4c2f-b133-9116df11b2d7" => "uhair_bald_plant_green",
		"eabc6671-5ffa-4d29-ad10-8741e992cb48" => "uhair_bald_plant_wilt",
		"21afb87c-e872-4f4c-af3c-c3685ed4e220" => "uhair_bald_skin",
		"07181400-e068-4e3c-b5c3-7c1c1c63948d" => "uhair_bald_sun",
		"13ae91e7-b825-4559-82a3-0ead8e8dd7fd" => "uhair_bald_vampire",
		"0e5c1d88-bf69-4be0-b828-b97f57596e6b" => "uhair_commercialmascot_fries",
		"26de136d-a5c4-49c7-a102-c4696858b818" => "uhair_cowmascot_holstein",
		"8567aaff-02b9-405c-b228-f2df976e9350" => "uhair_hatdeepseadiver",
		"d0d4b43b-33fe-4860-ae43-ab5c6d1672a3" => "uhair_hatlobotwitch_bad",
		"cfc7ce20-f2a3-4578-b28f-22a9cdde743f" => "uhair_hatlobotwitch_good",
		"22c8e9bb-a0b5-4d43-90b2-7d69a7de4469" => "uhair_hatlobotwitch_neutral",
		"320e7019-f2cf-41c7-800c-0cad8ec45eb0" => "uhair_hatmascotknightclose_ep7",
		"485c51af-3f24-4ca1-905c-73792b958b77" => "uhair_hatplantsimwitch_bad",
		"b54a2267-0e72-47f0-bd96-feefa739f1ab" => "uhair_hatplantsimwitch_good",
		"f9e51aa4-ae2c-4ed6-b926-6dbeb346a839" => "uhair_hatplantsimwitch_neutral",
		"d8ee896d-ccc7-4ce9-bb1b-2e34423c9a99" => "uhair_hatplantsimwitch_wilt_bad",
		"a55a4cd1-5c4f-4ddb-943b-040b05c56d4b" => "uhair_hatplantsimwitch_wilt_good",
		"41fc244c-ea3d-466c-925f-e9d542b8ff8d" => "uhair_hatplantsimwitch_wilt_neutral",
		"067602fe-e6e8-4510-92c9-ee6d203788f1" => "uhair_leafy_greens+whitepetals",
		"60ebdc5a-38ef-4f0e-b769-5e2983d39a15" => "uhair_leafy_wilt+wiltedpetals",
		"8076c5e8-108a-4307-ae75-7bb68c7be1ae" => "uhair_llamamascot",
		"d5f7826a-2667-410d-aa08-4ae890d8fda8" => "uhair_lycan",
		"00000000-0000-0000-0000-000000000000" => "uhair_null",
		"6c03ccda-ab68-4f46-8e94-908f8bf3f503" => "uhair_shocked",
		"d3fdf45e-ef95-4a8d-a3d2-759028a9cbc0" => "uhair_socialbunny_blue",
		"7591e79a-0f3c-476c-a44b-2b66ef53cf91" => "uhair_socialbunny_pink",
		"64d2502a-9958-4f6d-b2ac-de80a9879144" => "uhair_socialbunny_yellow",
		_ => "unknown"
	}).to_string()
}
