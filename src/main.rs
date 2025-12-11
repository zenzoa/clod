use std::error::Error;
use std::path::PathBuf;

use clap::{ Parser, Subcommand };

mod crc;
mod dbpf;
mod outfit;
mod defaulter;
mod extractor;
mod compressor;
mod bulk_edit;
mod recolor;

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
		source: Option<PathBuf>,
		/// Make the default replacement automatically without using the UI
		#[arg(short, long)]
		auto: bool,
		/// Hide pack icon in auto mode
		#[arg(short = 'p', long)]
		hide_pack_icon: bool
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
		/// Set whether hair is visible in CAS
		#[arg(short, long)]
		visible: Option<bool>,
		/// Set whether townies can use hair
		#[arg(short, long)]
		townified: Option<bool>,
		/// Set whether hair is a hat
		#[arg(short = 'H', long)]
		hat: Option<bool>,
		/// Use first family value for all hairs
		#[arg(short = 'f', long)]
		same_family: bool,
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
	/// Extracts makeup and unmeshed facial hair from game files for use in default replacements
	ExtractMakeup {
		/// Folder containing Skin.package files
		input: Option<PathBuf>,
		/// Folder to extract hair packages to
		#[arg(short, long, value_name="FOLDER")]
		output: Option<PathBuf>
	},
	/// Bulk edit GZPS properties in package files
	EditGZPS {
		/// List of package files to edit
		files: Vec<PathBuf>,
		/// GZPS property name
		#[arg(short, long)]
		property: String,
		/// New GZPS property value
		#[arg(short, long)]
		value: String
	},
	/// Compresses resources in package files
	Compress {
		/// List of package files to compress
		files: Vec<PathBuf>
	},
	/// Create one or more outfit recolors
	RecolorOutfit {
		/// One recolor package per desired age+gender to use as template
		files: Vec<PathBuf>,
		/// Title for recolors
		#[arg(short, long)]
		title: Option<String>,
		/// Number of new recolor packages to make
		#[arg(short, long)]
		number: Option<usize>,
		/// Repository recolors to first age+gender
		#[arg(short, long)]
		repo: bool
	}
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
	let args = Args::parse();
	match args.command {
		Some(Command::DefaultOutfit{ source, auto, hide_pack_icon }) => {
			defaulter::default_outfit::default_outfit(source, auto, hide_pack_icon)
		}
		Some(Command::DefaultHair{ source, output, add_ages, all_categories, visible, townified, hat, hide_pack_icon, same_family }) => {
			defaulter::default_hair::default_hair(source, output, add_ages, all_categories, visible, townified, hat, hide_pack_icon, same_family)
		}
		Some(Command::ExtractOutfits{ input, output }) => {
			extractor::extract_outfits::extract_outfits(input, output)
		}
		Some(Command::ExtractHairs{ input, output }) => {
			extractor::extract_hairs::extract_hairs(input, output)
		}
		Some(Command::ExtractMakeup{ input, output }) => {
			extractor::extract_makeup::extract_makeup(input, output)
		}
		Some(Command::EditGZPS{ files, property, value }) => {
			bulk_edit::edit_gzps(files, &property, &value)
		}
		Some(Command::Compress{ files }) => {
			compressor::compress_packages(files)
		}
		Some(Command::RecolorOutfit{ files, title, number, repo }) => {
			recolor::recolor_outfit(files, title, number, repo)
		}
		None => Err("No command given.".into())
	}
}
