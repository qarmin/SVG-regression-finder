## SVG regression tester

This repository contains tool to automatically find if ThorVG, Inkscape or any other SVG converter, produce valid png files.

## How it works
By default, Inkscape and ThorVG produce png files from svg ones.  
Next this two files are compared with each other, and if are similar, then nothing happens, but if are different, then names of this files are printed to output.  
Most of files differs on shadow size and other very small visible elements, so it is possible to set similarity level, which should part of such files ignore.

## Can it work with X, Y or Z tool/library?
Yes, changes are very simple.  
To add support for different library like rsvg, just replace logic of running inkscape or thorvg with own tool and everything should work fine.

## Current Requirements
Current Requirements:
- Inkscape installed - any version, but newer should create better png files
- [ThorVG](https://github.com/Samsung/thorvg/) - the best is to compile it manually, since it is fast and quite simple

## Usage
- Compile thorvg with svg2png tool - `meson  -Dtools=svg2png . build;ninja -C build/` should work 
- Compile app `cargo build --release` or download prebuilt binaries(not sure if are available)
- Create txt file e.g. `input.txt` with full paths to svg files to check, one per line e.g.
```
/home/korzyk/a.svg
/home/korzyk/b.svg
/home/korzyk/c.svg
```
**Warning**  
for `/home/korzyk/a.svg` this will create in same folder two additional files `/home/korzyk/a_inkscape.png` and `/home/korzyk/a_thorvg.png`, so I suggest to move this SVG files to its own empty folder.
- Be sure that inkscape is available by typing `inkscape`
- Run app, command explanation
```
app <path_to_svg2png> <path_to_files_to_check> <size_of_created_images> <similarity>
```
example usage
```
app "/home/rafal/test/thorvg/build/src/bin/svg2png/svg2png" "input.txt" 500 10
```
```
<path_to_svg2png> - path to thorvg svg2png tool
<path_to_files_to_check> - path to file with full paths in each line(like in example from above)
<size_of_created_images> - width and height of created images, value 500 means that to test, thorvg and inkscape will create from svg files, png ones with size 500x500
<similarity> - difference between images that will be ignored. 0 could find very small differences, line e.g. line different by 1px, so I use 10 which is I think optimal. When using bigger value, only files which differ a lot will be visible
```
- read output of command
```
	firefox /home/rafal/Desktop/SVGTEST/ABCD_4021.svg; firefox /home/rafal/Desktop/SVGTEST/ABCD_4021_inkscape.png; firefox /home/rafal/Desktop/SVGTEST/ABCD_4021_thorvg.png
	firefox /home/rafal/Desktop/SVGTEST/ABCD_4024.svg; firefox /home/rafal/Desktop/SVGTEST/ABCD_4024_inkscape.png; firefox /home/rafal/Desktop/SVGTEST/ABCD_4024_thorvg.png
	firefox /home/rafal/Desktop/SVGTEST/ABCD_11041.svg; firefox /home/rafal/Desktop/SVGTEST/ABCD_11041_inkscape.png; firefox /home/rafal/Desktop/SVGTEST/ABCD_11041_thorvg.png
	firefox /home/rafal/Desktop/SVGTEST/ABCD_12046.svg; firefox /home/rafal/Desktop/SVGTEST/ABCD_12046_inkscape.png; firefox /home/rafal/Desktop/SVGTEST/ABCD_12046_thorvg.png
```
- Now you can simply copy/paste this commands into commandline to open multiple files in firefox to be able quite easily compare them by changing tabs. I used firefox instead e.g. `xdg-open`, because eog and eom started to crash constantly when opening several images
- Copy possibly broken files to new location I prepared small python script, available in `src/python_renamer.py` which will convert firefox commands from above to cp, to be able to easily copy this files to new folder
- Check manually differences