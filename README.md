# clod
Command Line Outfit Defaulter (and multi-tool) for The Sims 2

## Extract Original TS2 Outfit Templates
- Copy any `Skins.package` in your TS2 install into their own folder. (You'll need to rename them of course, since they all have the same file name!)
- Create a folder called `output` to store all the package files CLOD will generate.
- Open a terminal.
- Run CLOD with the path to the folder with your Skins files (unless that's the folder you're already in) and the path to the output folder. For example: `clod extract-outfits ./skins -o ./skins/output`
- In your `output` folder, you'll find a ton of `.package` files labeled with the ages, genders, type, and name of all the outfits in your game. Each contains the 3IDR and GZPS resources necessary to make default replacements.

## Extract Original TS2 Hair Templates
- Follow the same instructions for extracting outfits, but run the `extract-hairs` command instead. For example: `clod extract-hairs ./skins -o ./skins/output_hairs`
- CLOD will create one folder per hair family, except where a single hair (like `fhairaline`) uses a different family for each age (in which case it combines them in one folder). Hairs that are normally hidden will have `HIDDEN` in the file name, and hairs that include duplicate ages (like young adult clones when the adult version is enabled for young adults) are disabled with a `.off` extension.
- Hats usually have two sets of hair files: one for the hat itself, and one for the hatless state. The hatless files are hidden clones that link to the meshes and textures of other hairs in the game. Because it's common to replace hats with regular hair, CLOD disables the hatless files with a `.off` extension (except when a particular age doesn't have a companion hat file). If you want to replace the hatless hairs separately, move them into their own folder and remove the `.off` extensions.

## Create TS2 Clothing Default Replacements (UI)

### Step 1: Setup Your Files
- Create a folder that contains the original outfits. That is, the `.package` file(s) containing the the 3IDR + GZPS resources for the outfits you want to replace.
- Inside this folder, place one or more subfolders with your replacement outfits. That is, the mesh and recolor `.package` files for the new outfits you want to use. For example:
```
|-- witches
    |-- af_body_witch.package
    |-- ef_body_witch.package
    |-- tf_body_witch.package
	|
    |-- SalemAF
    |   |-- SalemAF_MESH.package
    |   |-- SalemAF_black.package
    |   |-- SalemAF_green.package
    |   |-- SalemAF_white.package
	|
    |-- SalemEF_REPO
    |   |-- SalemEF_MESH.package
    |   |-- SalemEF_black_REPO.package
    |   |-- SalemEF_green_REPO.package
    |   |-- SalemEF_white_REPO.package
	|
    |-- SalemTF_REPO
        |-- SalemTF_MESH.package
        |-- SalemTF_black_REPO.package
        |-- SalemTF_green_REPO.package
        |-- SalemTF_white_REPO.package
```

### Step 2: Launch CLOD
- Open a terminal
- Launch CLOD with the path to the folder containing your outfits. For example: `clod default-outfit ./witches`

### Step 3: Choose Replacements + Properties
You can navigate CLOD with the mouse or with the keyboard (use arrow keys to navigate focus and ENTER to select).
- The list of original outfits is on the left.
- The properties for the selected outfit are on the right.
- For each original outfit you want to replace, select a replacement outfit from the dropdown at the top of the properties panel. Change the flags, categories, genders, and ages as you see fit.
- NOTE: Any outfit without a replacement won't be included in the output file.

### Step 4: Save + Test
- Click SAVE at the bottom-right.
- Change the output filename is you want.
- You can choose whether you want to hide the pack icon in CAS and/or compress resources.
- Click OK and wait for your file to save.
- Et voila! You should find a file named something like `DEFAULT.package` in your original folder.
- Place this file in your TS2 Downloads folder, and launch TS2 to test your default replacement.
- To update the thumbnails, remove `[TS2 Documents folder]/Thumbnails/CASThumbnails.package`.

## Create TS2 Clothing Default Replacements (Auto)

### Step 1: Setup Your Files
- Same as above.

### Step 2: Create Properties File
- Create an empty file in the folder with the original outfits, and give it the extention `.properties`. This could just be a renamed text file, for example - CLOD won't read the file contents, just the file name.
- The name of the file should consist of a series of tags, eg. `unisex_everyday_formal_notownies.properties`. Here are the tags you can use:
	- `unisex`: Enable for both genders, if the outfit is for babies, toddlers, and children. If absent, use the original genders.
	- `hidden`: Hide in CAS. If absent, show in CAS.
	- `notownies`: Disable for townies. If absent, enable for townies.
	- `everyday` or `casual`: Enable in everyday category. If absent, disable in that category.
	- `swim`: Enable in swimwear category. If absent, disable in that category.
	- `sleep` or `pjs`: Enable in sleepwear category. If absent, disable in that category.
	- `formal` or `fancy`: Enable in formalwear category. If absent, disable in that category.
	- `underwear` or `undies`: Enable in underwear category. If absent, disable in that category.
	- `maternity` or `pregnant`: Enable in maternity category. If absent, disable in that category.
	- `active` or `athletic`: Enable in activewear category. If absent, disable in that category.
	- `outerwear`: Enable in outerwear category. If absent, disable in that category.
- If there is no properties file in the folder, CLOD will just use the default properties.

### Step 3: Launch CLOD
- Open a terminal
- Launch CLOD with the path to the folder containing your outfits with the `-a/--auto` parameter. For example: `clod default-outfit ./witches -a`
- You can remove the pack icon from CAS by setting the `product` setting to 1 with the `-p/--hide-pack-icon` parameter:  `clod default-outfit ./witches -ap`
- The default replacements will be saved to the same folder using the name of the folder for the filename. For example: `witches_DEFAULT.package`
- Any extra clothes not used in the default replacement will be placed in a `_EXTRAS` file containing decustomized outfits repo'd to the default replacements. For example: `witches_EXTRAS.package`

## Create TS2 Hair Default Replacements

### Step 1: Setup Your Files
- Create a folder that contains the original hair files. That is, the `.package` file(s) containing the the 3IDR + GZPS resources for the outfits you want to replace.
- Inside this folder, place your replacement hair files in a subfolder. That is, the mesh and recolor `.package` files for the new hair you want to use. For example:
```
|-- fhair_poofs
    |-- efhair_poofs_grey.package
	|
    |-- ayfhair_poofs_black.package
    |-- ayfhair_poofs_blond.package
    |-- ayfhair_poofs_brown.package
    |-- ayfhair_poofs_red.package
	|
    |-- tfhair_poofs_black.package
    |-- tfhair_poofs_blond.package
    |-- tfhair_poofs_brown.package
    |-- tfhair_poofs_red.package
	|
	|-- ...
	|
    |-- platasp_dogsill_AJBuns
        |-- platasp_dogsill_AJBuns_MESH.package
        |-- platasp_dogsill_AJBuns_black.package
        |-- platasp_dogsill_AJBuns_blond.package
        |-- platasp_dogsill_AJBuns_brown.package
        |-- platasp_dogsill_AJBuns_red.package
```

### Step 2: Launch CLOD
- Open a terminal
- Launch CLOD with the path to the folder containing your hairs. For example: `clod default-hair ./fhair_poofs`.
- By default this will automatically replace each hair file in the main with a matching age/color from the replacement subfolder, and save the output to `./fhair_poofs/fhair_poofs_DEFAULT.package`.
- You can specify a different path for the output with the `-o/--output` parameter: `clod default-hair ./fhair_poofs -o ./mypoofsdefault.package`
- If the replacement hair has more ages than the original, you can add those extra ages as decustomized and linked with the `-a/--add-ages` parameter: `clod default-hair ./fhair_poofs -a`
- To enable the hair for all categories, add the `-c/--all-categories` parameter: `clod default-hair ./fhair_poofs -c`
- To override the original hair's flags for CAS visibility, townification, or hat status*, use the `-v/--visible`, `-t/--townified`, and `-H/--hat` parameters: `clod default-hair ./fhair_poofs -v true -t false -H false`
- For hairs that use multiple families (like `fhairaline`) you can force the replacements to use the same family (whatever family used by the first file) with the `-f/--same-family` parameter: `clod default-hair ./fhair_poofs -f`
- You can remove the pack icon from CAS by setting the `product` setting to 1 with the `-p/--hide-pack-icon` parameter: `clod default-hair ./fhair_poofs -p`

\* Note on hats: If you want sims to revert to another default replacement when they remove the hat, you'll have to link the hatless hidden clones manually. But if you're replacing one of the original hats with a regular hair and don't want the sim to remove it, you can just turn off the hat flag and ignore the hatless hidden clones.

## Compress TS2 Package Files
WARNING: Highly experimental!
- Open a terminal and navigate to the folder containing the package file(s) you want to compress.
- To compress one file, run CLOD like this: `clod compress [filename]`
- To compress multiple files, run CLOD like this: `clod compress [filename1] [filename2] [filename3]`
- To compress all package files in the folder, run CLOD like this: `clod compress *.package`
- The original file will be backed up with the extension `.package.bak`.

## Create TS2 Outfit Recolors
- Open a terminal and navigate to the folder containing the outfit package file(s) you want to recolor.
- Launch CLOD with the file name of one of the existing recolors, which will be used as a template. Use the `-t/--title` parameter to give your recolors a name, and the `-n/--number` to tell CLOD how many new recolors to make. For example: `clod recolor-outfit ./SalemAF_black.package -n 3 -t "my_Salem"` will generate the files my_SalemAF_01.package, my_SalemAF_02.package, and my_SalemAF_03.package.
- You can make recolors for multiple ages at once by specifying multiple file names. For example: `clod recolor-outfit ./SalemAF_black.package ./SalemTF_black.package ./SalemEF_black.package -n 1 -t "my_Salem"` will generate the files my_SalemAF_01.package, my_SalemTF_01.package, and my_SalemEF_01.package.
- To make the additional recolors repositoried to the first set, use the `-r/--repo` parameter. For example: `clod recolor-outfit ./SalemAF_black.package ./SalemTF_black.package ./SalemEF_black.package -n 1 -t "my_Salem" -r` will generate the files my_SalemAF_01.package, my_SalemTF_01_REPO.package, and my_SalemEF_01_REPO.package.
- You will still need to replace the textures in an external program, like YAPE or SimPE.
