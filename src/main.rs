use std::error::Error;
use std::path::{ Path, PathBuf };

use clap::{ Parser, Subcommand };

mod crc;
mod dbpf;
mod outfit;
mod defaulter;
mod extractor;

use defaulter::DefaultSettings;

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
		/// Package containing original outfit(s)' GZPS and 3IDR resources
		original: PathBuf,
		/// Folder containing replacement mesh and recolor packages
		#[arg(short, long, value_name="FOLDER")]
		replacements: Option<PathBuf>,
		/// Compress resources
		#[arg(short, long)]
		compress: bool,
		/// Ignore missing mesh/texture resources
		#[arg(short, long)]
		ignore_missing: bool,
		/// Enable baby, toddler, and child outfits for all genders
		#[arg(short, long)]
		gender_fix: bool,
		/// Set product ids to Base Game to remove pack icon
		#[arg(short, long)]
		product_fix: bool,
		/// Set flags to 0 to make outfit visible in CAS and wearable by townies
		#[arg(short, long)]
		flag_fix: bool,
	},
	/// Extracts outfits from game files for use in default replacements
	ExtractOutfits {
		/// Folder containing Skin.package files
		input: Option<PathBuf>,
		/// Folder to extract outfit packages to
		#[arg(short, long, value_name="FOLDER")]
		output: Option<PathBuf>
	}
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
	let args = Args::parse();
	match args.command {
		Some(Command::DefaultOutfit{ original, replacements, compress, ignore_missing, gender_fix, product_fix, flag_fix }) => {
			let default_folder = original.parent().unwrap_or(Path::new("./")).to_path_buf();
			defaulter::default_outfit(
				&original,
				&replacements.unwrap_or(default_folder),
				DefaultSettings{ compress, ignore_missing, gender_fix, product_fix, flag_fix }
			)
		}
		Some(Command::ExtractOutfits{ input, output }) => {
			let input_folder = input.unwrap_or(PathBuf::from("./"));
			let default_output = input_folder.parent().unwrap_or(Path::new("./")).to_path_buf();
			extractor::save_skins(
				&input_folder,
				&output.unwrap_or(default_output)
			)
		}
		None => Err("No command given.".into())
	}
}
