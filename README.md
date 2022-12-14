## SVG regression tester

This repository contains tool to automatically find if ThorVG, Rsvg, Inkscape or any other SVG converter, produce valid png files.  
It can be used to check visually differences between two versions of same software.

## How it works
By default, Rsvg and ThorVG produce png files from svg ones.  
Next this two files are compared with each other, and if are similar, then nothing happens, but if are different, then names of this files are printed to output.  
Most of files differs on shadow size and other very small visible elements, so it is possible to set similarity level, which should part of such files ignore.

## Can it work with X, Y or Z tool/library?
Yes, changes are very simple and require to add/modify fields structure inside main.rs file.

## Current Requirements
Requirements:
- Rsvg installed - on Ubuntu can be installed via `sudo apt install librsvg2-bin`
- [ThorVG](https://github.com/Samsung/thorvg/) - the best is to compile it manually, since it is fast and quite simple

Optional dependencies:
- Inkscape - it is possible to use inkscape instead rsvg, but this requires to enable some code directly in src/main.rs

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
for `/home/korzyk/a.svg` this will create in same folder two additional files `/home/korzyk/a_rsvg.png` and `/home/korzyk/a_thorvg.png`, so I suggest to move this SVG files to its own empty folder.
- Be sure that rsvg is available by typing `rsvg-convert --help`
- Run app, command explanation
```
app <path_to_svg2png> <path_to_files_to_check> <size_of_created_images> <similarity>
```
example usage
```
app "/home/rafal/test/thorvg/build/src/bin/svg2png/svg2png" "input.txt" 500 10
```
or
```
app "/home/rafal/test/thorvg/build/src/bin/svg2png/svg2png" "/home/folder" 500 10
```
```
<path_to_svg2png> - path to thorvg svg2png tool
<path_to_files_to_check> - path to file with full paths in each line(like in example from above) or path to folder from which are taken only files 1 folder depth
<size_of_created_images> - width and height of created images, value 500 means that to test, thorvg and rsvg will create from svg files, png ones with size 500x500
<similarity> - difference between images that will be ignored. 0 could find very small differences, line e.g. line different by 1px, so I use 10 which is I think optimal. When using bigger value, only files which differ a lot will be visible
```
- read output of command
```
	firefox /home/rafal/Desktop/SVGTEST/ABCD_4021.svg; firefox /home/rafal/Desktop/SVGTEST/ABCD_4021_rsvg.png; firefox /home/rafal/Desktop/SVGTEST/ABCD_4021_thorvg.png
	firefox /home/rafal/Desktop/SVGTEST/ABCD_4024.svg; firefox /home/rafal/Desktop/SVGTEST/ABCD_4024_rsvg.png; firefox /home/rafal/Desktop/SVGTEST/ABCD_4024_thorvg.png
	firefox /home/rafal/Desktop/SVGTEST/ABCD_11041.svg; firefox /home/rafal/Desktop/SVGTEST/ABCD_11041_rsvg.png; firefox /home/rafal/Desktop/SVGTEST/ABCD_11041_thorvg.png
	firefox /home/rafal/Desktop/SVGTEST/ABCD_12046.svg; firefox /home/rafal/Desktop/SVGTEST/ABCD_12046_rsvg.png; firefox /home/rafal/Desktop/SVGTEST/ABCD_12046_thorvg.png
```
- Now you can simply copy/paste this commands into commandline to open multiple files in firefox to be able quite easily compare them by changing tabs. I used firefox instead e.g. `xdg-open`, because eog and eom started to crash constantly when opening several images
- Copy possibly broken files to new location I prepared small python script, available in `src/python_renamer.py` which will convert firefox commands from above to cp, to be able to easily copy this files to new folder
- Check manually differences

Example differences that found this tool(in Japanese flag look at shadows inside red circle)

![Screenshot from 2022-08-26 16-25-58](https://user-images.githubusercontent.com/41945903/186930569-0c46657c-9054-42e0-9eb4-a539b6eccbe4.png)

## Limitations
For now it cannot check non square svg files, because ThorVG unlike Inkscape or Rsvg resize them in different way