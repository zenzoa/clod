# clod
Command Line Outfit Defaulter (and multi-tool) for The Sims 2

## Extract Original TS2 Outfit/Hair Templates
- Copy any `Skins.package` in your TS2 install into their own folder. (You'll need to rename them of course, since they all have the same file name!)
- Optional: do the same thing with `globalcatbin.bundle.package` files, in a separate folder.
- Create a folder called `output` to store all the package files CLOD will generate.
- Open a terminal and run CLOD with paths for your skins, bins, and output folders. For example: `clod extract-outfits ./skins --bins ./bins --output ./output` (for hairs, replace `extract-outfits` with `extract-hairs`)
- In your `output` folder, you'll find a ton of `.package` files containing the 3IDR and GZPS resources (and BINX resources if including bins) necessary to make default replacements.

## Extract Original TS2 Objects
- Create a folder for the extracted object package files to get saved to.
- Open a terminal and run CLOD with paths for your TS2 install directory and your output folder. For example: `clod extract-objects "~/.wine/drive_c/Program Files/The Sims 2" --output ./objects`
- In your `output` folder, you'll find a ton of `.package` files organized by function category, each containing certain key resources necessary for making default replacements and CEP packages, eg. MMAT, TXMT, TXTR, LIFO, GMND, GMDC, SHPE, etc.

## Create Outfit Default Replacements
- Here's how I set up my folders: once I've extracted my outfits, I pick an outfit I want to replace and go into the folder containing the templates for that outfit. There, I put the folder containing the replacement outfits.
- If I don't want to use the original outfit's flags and categories, I create an empty text file that I name (for example) `unisex_hidden_notownies_everyday_formal.properties` (adding or removing flags and categories based on what I want the output to be). Tags `unisex` and `pregnant` are only applied to outfits of appropriate ages.
- Create an output folder for the default replacement files to go.
- Open a terminal, `cd` into the folder with the original outfit, and run CLOD with the paths for original, replacement, and output folders. For example: `clod default-outfit ./ --replacement ./sg_AFunMashupAF --output ./output --include_extras`
- The optional `include_extras` flag puts any additional ages, genders, and recolors in a separate package file. They'll use the flags/categories set in the `.properties` file (if it exists), they'll be decustomized (they won't get the custom star in the catalog), and they'll be sorted alongside the default replacements. If they share any resources with the ones used for the default replacement, they'll be repositoried to them (in other words, the EXTRAS package will require the DEFAULT package).
- Note: You can replace multiple outfits with the same replacement by putting the template packages in the same input folder. CLOD adds BINX resources to all its default replacements to make sure all replaced outfits are sorted together, and to make sure hidden clones show up in the catalog.

## Create Hair Default Replacements
- Same process as above, with the addition of the `fallback` argument that lets you define a path to a second replacement hair. Except this one will only get used for any ages/genders/colors that aren't covered by the first replacement, and the resources won't get included in the output package, only repo'd. Useful for making sure hidden clones "link" to hairs you like.
- For example: `clod default-hair ./ --replacement ./sg_WhatAHairAF --fallback ./sg_HairForAllAges --output ./output --include_extras`

## Recolor an Outfit
- Open a terminal and run CLOD with the path to the outfit mesh package you want to recolor. For example: `clod recolor-outfit sg_StripedSweaterEmbroideredShortsAF_MESH.package -o sg_StripedSweaterEmbroideredShortsAF -t "sg_StripedSweaterEmbroidredShortsAF" -n "sg_stripesweaterembroideredshorts_af" -a adult -g female -s normal -c everyday -p body -L -m 3`
- This one has a ton of arguments so let's go through them:
	- `-o <path>` or `--output <path>`: Base name for recolor package(s) (leave off ".package")
	- `-m <number>` or `--multiple <number>`: How many recolors to make. Default is 1.
	- `-t <string>` or `--tooltip <string>`: What the tooltip should be.
	- `-n <string>` or `--name <string>`: Name (not seen in game, but used to name resources and included in the GZPS)
	- `-a <string>` or `--age <string>`: Use this argument multiple times to include multiple ages. Possible values: "p"/"toddler", "c(hild)", "t(een)", "y(oungadult)", "a(dult)", "e(lder)"
	- `-g <string>` or `--gender <string>`: Possible values: "f(emale)", "m(ale)", "u(nisex)"
	- `-c <string>` or `--category <string>`: Use this argument multiple times to include multiple categories. Possible values: "e(veryday)", "f(ormal)", "u(nderwear)", "p(ajamas)", "s(wim)", "a(ctive)", "o(uterwear)", "P(regnant)"
	- `-s <string>` or `--shoe <string>`: Possible values: "n(one)", "d(efault)"/"normal", "b(barefoot)", "B"/"boots", "h(eels)", "s(andals)", "p(ajamas)", "a(rmor)"
	- `-p <string>` or `--part <string>`: Possible values: "f(ullbody)"/"body", "t(op)", "b(ottom)"
	- `-f <string>` or `--flags <string>`: Possible values: "h(idden)", "t"/"notownies", "w"/"noworkers", "d(efault)"
	- `-S <number>` or `--sort <number>`: Sort value used in BINX.
	- `-T <string>` or `--textureless <string>`: Optional. Name of subset that should not use a texture. Use this argument multiple times to include multiple subsets.
	- `-L` or `--l`: Optional. List TXMTs last in the 3IDR instead of first, Latmos-style.

### Repositoried Recolors
- You can make repo'd outfit recolors by including additional paths after the mesh package and using the `-r`/`--repo` argument. For example: `clod recolor-outfit sg_StripedSweaterEmbroideredShortsTF_MESH.package sg_StripedSweaterEmbroideredShortsAF_0* -o sg_StripedSweaterEmbroideredShortsTF -t "sg_StripedSweaterEmbroidredShortsTF" -n "sg_stripesweaterembroideredshorts_tf" -a teen -g female -s normal -c everyday -p body -T body -L --repo`
