# clod
Command Line Outfit Defaulter (and multi-tool) for The Sims 2

## Extract Original TS2 Outfit Templates
- Copy any `Skins.package` in your TS2 install into their own folder. (You'll need to rename them of course, since they all have the same file name!)
- Create a folder called `output` to store all the package files CLOD will generate.
- Open a terminal.
- Run CLOD with the path to the folder with your Skins files (unless that's the folder you're already in) and the path to the output folder. For example: `clod extract-outfits ./skins -o ./skins/output`
- In your `output` folder, you'll find a ton of `.package` files labeled with the ages, genders, type, and name of all the outfits in your game. Each contains the 3IDR and GZPS resources necessary to make default replacements.

## Create TS2 Clothing Default Replacements

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

## Compress TS2 Package Files
- Open a terminal and navigate to the folder containing the package file(s) you want to compress.
- To compress one file, run CLOD like this: `clod compress [filename]`
- To compress multiple files, run CLOD like this: `clod compress [filename1] [filename2] [filename3]`
- To compress all package files in the folder, run CLOD like this: `clod compress *.package`
- The original file will be backed up with the extension `.package.bak`.
