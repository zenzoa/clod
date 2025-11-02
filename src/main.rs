use std::error::Error;
use std::path::PathBuf;

use clap::{ Parser, Subcommand };

mod crc;
mod dbpf;
mod outfit;
mod defaulter;
mod extractor;
mod compressor;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
	#[command(subcommand)]
	command: Option<Command>
}

#[derive(Subcommand)]
enum Command {
	/// Generates a default replacement for a TS2 outfit
	DefaultOutfit {
		/// Folder containing original outfits, and subfolder(s) containing replacements
		source: Option<PathBuf>
	},
	/// Generates a default replacement for a TS2 outfit
	DefaultHair {
		/// Folder containing original hairs, and subfolder(s) containing replacements
		source: Option<PathBuf>,
		/// Path for default replacement package
		#[arg(short, long)]
		output: Option<PathBuf>,
		/// Add ages from replacement hair, even if not included in original hair
		#[arg(short, long)]
		add_ages: bool,
		/// Enable for all categories
		#[arg(short = 'c', long)]
		all_categories: bool,
		/// Set flags property to new value
		#[arg(short, long)]
		flags: Option<u32>,
		/// Set family property to new value
		#[arg(short = 'F', long)]
		family: Option<String>,
		/// Hide pack icon
		#[arg(short = 'p', long)]
		hide_pack_icon: bool
	},
	/// Extracts outfits from game files for use in default replacements
	ExtractOutfits {
		/// Folder containing Skin.package files
		input: Option<PathBuf>,
		/// Folder to extract outfit packages to
		#[arg(short, long, value_name="FOLDER")]
		output: Option<PathBuf>
	},
	/// Extracts hairs from game files for use in default replacements
	ExtractHairs {
		/// Folder containing Skin.package files
		input: Option<PathBuf>,
		/// Folder to extract hair packages to
		#[arg(short, long, value_name="FOLDER")]
		output: Option<PathBuf>
	},
	/// Finds all hairs with a given family value
	FindHairs {
		/// Folder containing Skin.package files
		input: Option<PathBuf>,
		/// Family property
		#[arg(short, long, value_name="ID")]
		family: String
	},
	/// Compresses resources in package files
	Compress {
		/// List of package files to compress
		files: Vec<PathBuf>
	}
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
	let args = Args::parse();
	match args.command {
		Some(Command::DefaultOutfit{ source }) => {
			defaulter::default_outfit::default_outfit(source)
		}
		Some(Command::DefaultHair{ source, output, add_ages, all_categories, hide_pack_icon, flags, family }) => {
			defaulter::default_hair::default_hair(source, output, add_ages, all_categories, hide_pack_icon, flags, family)
		}
		Some(Command::ExtractOutfits{ input, output }) => {
			extractor::extract_outfits(input, output)
		}
		Some(Command::ExtractHairs{ input, output }) => {
			extractor::extract_hairs(input, output)
		}
		Some(Command::FindHairs{ input, family }) => {
			extractor::find_hairs(input, family)
		}
		Some(Command::Compress{ files }) => {
			compressor::compress_packages(files)
		}
		None => Err("No command given.".into())
	}
}
