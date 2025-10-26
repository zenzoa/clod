use std::error::Error;
use std::path::{ Path, PathBuf };

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
	/// Extracts outfits from game files for use in default replacements
	ExtractOutfits {
		/// Folder containing Skin.package files
		input: Option<PathBuf>,
		/// Folder to extract outfit packages to
		#[arg(short, long, value_name="FOLDER")]
		output: Option<PathBuf>
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
			defaulter::default_outfit(source)
		}
		Some(Command::ExtractOutfits{ input, output }) => {
			let input_folder = input.unwrap_or(PathBuf::from("./"));
			let default_output = input_folder.parent().unwrap_or(Path::new("./")).to_path_buf();
			extractor::save_skins(
				&input_folder,
				&output.unwrap_or(default_output)
			)
		}
		Some(Command::Compress{ files }) => {
			compressor::compress_packages(files)
		}
		None => Err("No command given.".into())
	}
}
