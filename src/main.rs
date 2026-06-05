use std::error::Error;
use std::path::PathBuf;

use clap::{ Parser, Subcommand };

mod helpers;
mod crc;
mod dbpf;
mod outfit;
mod hair;
mod object;
mod extractor;

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
		/// Folder containing original outfit template packages
		original: PathBuf,
		/// Folder containing replacement outfit packages
		#[arg(short, long)]
		replacement: Option<PathBuf>,
		/// Output file path
		#[arg(short, long)]
		output: Option<PathBuf>,
		/// Include extra ages/genders/recolors as decustomized and repo'd
		#[arg(short = 'e', long)]
		include_extras: bool,
		/// Suppress non-error/non-warning output
		#[arg(short, long)]
		quiet: bool
	},

	/// Generates a default replacement for a TS2 hair
	DefaultHair {
		/// Folder containing original hair template packages
		original: PathBuf,
		/// Folder containing replacement hair packages
		#[arg(short, long)]
		replacement: Option<PathBuf>,
		/// Folder containing fallback hair packages that will be referenced (but not included) for hidden clones without a replacement
		#[arg(short, long)]
		fallback: Option<PathBuf>,
		/// Output file path
		#[arg(short, long)]
		output: Option<PathBuf>,
		/// Include extra ages/genders/colors as decustomized and repo'd
		#[arg(short = 'e', long)]
		include_extras: bool,
		/// Suppress non-error/non-warning output
		#[arg(short, long)]
		quiet: bool
	},

	/// Extracts object sources from game files for use in default replacements
	ExtractObjects {
		/// Sims 2 installation directory
		input: Option<PathBuf>,
		/// Folder to extract object packages to
		#[arg(short, long, value_name="FOLDER")]
		output: Option<PathBuf>
	},

	/// Extracts outfits from game files for use in default replacements
	ExtractOutfits {
		/// Folder containing Skins.package files
		input: Option<PathBuf>,
		/// Folder to extract outfit packages to
		#[arg(short, long, value_name="FOLDER")]
		output: Option<PathBuf>,
		/// Folder containing globalcatbin.bundle.package files
		#[arg(short, long, value_name="FOLDER")]
		bins: Option<PathBuf>
	},

	/// Extracts hairs from game files for use in default replacements
	ExtractHairs {
		/// Folder containing Skin.package files
		input: Option<PathBuf>,
		/// Folder to extract hair packages to
		#[arg(short, long, value_name="FOLDER")]
		output: Option<PathBuf>,
		/// Folder containing globalcatbin.bundle.package files
		#[arg(short, long, value_name="FOLDER")]
		bins: Option<PathBuf>
	},

	/// Create one or more outfit recolors
	RecolorOutfit {
		/// Package file for outfit to recolor (mesh or existing recolor)
		files: Vec<PathBuf>,
		/// Output file path
		#[arg(short, long)]
		output: Option<PathBuf>,
		/// Create multiple recolors
		#[arg(short, long)]
		multiple: Option<usize>,
		/// Repository recolor(s) to any additional files given as arguments
		#[arg(short, long)]
		repo: bool,
		/// Tooltip text
		#[arg(short, long)]
		tooltip: Option<String>,
		/// Name (not seen in game)
		#[arg(short, long)]
		name: Option<String>,
		/// Age(s) ("p"/"toddler", "c(hild)", "t(een)", "y(oungadult)", "a(dult)", "e(lder)")
		#[arg(short, long)]
		age: Option<Vec<String>>,
		/// Gender ("f(emale)", "m(ale)", "u(nisex)")
		#[arg(short, long)]
		gender: Option<String>,
		/// Category ("e(veryday)", "f(ormal)", "u(nderwear)", "p(ajamas)", "s(wim)", "a(ctive)", "o(uterwear)", "P(regnant)")
		#[arg(short, long)]
		category: Option<Vec<String>>,
		/// Shoe sound ("n(one)", "d(efault)"/"normal", "b(barefoot)", "B"/"boots", "h(eels)", "s(andals)", "p(ajamas)", "a(rmor)")
		#[arg(short, long)]
		shoe: Option<String>,
		/// Part ("f(ullbody)"/"body", "t(op)", "b(ottom)")
		#[arg(short, long)]
		part: Option<String>,
		/// Flags ("h(idden)", "t"/"notownies", "w"/"noworkers", "d(efault)")
		#[arg(short, long)]
		flags: Option<Vec<String>>,
		/// Sort index
		#[arg(short='S', long)]
		sort: Option<i32>,
		/// Textureless subsets
		#[arg(short='T', long)]
		textureless: Option<Vec<String>>,
		/// List TXMTs last in the 3IDR
		#[arg(short='L', long)]
		txmts_last: bool,
	},

	/// Create one or more object recolors
	RecolorObject {
		/// Package file for the object you want to recolor
		file: PathBuf,
		/// Output file path
		#[arg(short, long)]
		output: Option<PathBuf>,
		/// Title for recolors
		#[arg(short, long)]
		name: Option<String>,
		/// Specify subset to recolor; otherwise recolors will include all subsets
		#[arg(short, long)]
		subset: Option<String>,
		/// Create multiple recolors
		#[arg(short, long)]
		multiple: Option<usize>
	},

	/// Create a new object recolor based on an existing one
	CloneRecolor {
		/// Package file for the recolor you want to clone
		file: PathBuf,
		/// Output file path
		#[arg(short, long)]
		output: Option<PathBuf>,
		/// Part of the original name (in mmats, txmts, and txtrs) you want to replace
		#[arg(short='p', long)]
		old_name: String,
		/// What to replace old name with
		#[arg(short, long)]
		new_name: String,
		/// Create multiple recolors
		#[arg(short, long)]
		multiple: Option<usize>
	},
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
	let args = Args::parse();
	match args.command {
		Some(Command::DefaultOutfit{ original, replacement, output, include_extras, quiet }) => {
			outfit::default_outfit::default_outfit(original, replacement, output, include_extras, quiet)
		}

		Some(Command::DefaultHair{ original, replacement, fallback, output, include_extras, quiet }) => {
			hair::default_hair::default_hair(original, replacement, fallback, output, include_extras, quiet)
		}

		Some(Command::ExtractObjects{ input, output }) => {
			extractor::extract_objects::extract_objects(input, output)
		}

		Some(Command::ExtractOutfits{ input, output, bins }) => {
			extractor::extract_outfits::extract_outfits(input, output, bins)
		}

		Some(Command::ExtractHairs{ input, output, bins }) => {
			extractor::extract_hairs::extract_hairs(input, output, bins)
		}

		Some(Command::RecolorOutfit { files, output, multiple, repo, tooltip, name, age, gender, category, shoe, part, flags, sort, textureless, txmts_last }) => {
			outfit::recolor_outfit::recolor_outfit(files, output, multiple, repo, tooltip, name, age, gender, category, shoe, part, flags, sort, textureless, txmts_last)
		},

		Some(Command::RecolorObject { file, output, name, subset, multiple }) => {
			object::recolor_object::recolor_object(file, output, name, subset, multiple)
		},

		Some(Command::CloneRecolor { file, output, old_name, new_name, multiple }) => {
			object::recolor_object::clone_recolor(file, output, old_name, new_name, multiple)
		},

		None => Err("No command given.".into())
	}
}
