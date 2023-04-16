## SVG regression tester

This repository contains tool to automatically find if ThorVG, Rsvg, Inkscape or any other SVG converter, produce valid png files.  
It can be used to check visually differences between two versions of same software.

## How it works
By default, Rsvg and ThorVG produce png files from svg ones.  
Next this two files are compared with each other, and if are similar, then nothing happens, but if are different, then names of this files are printed to output.  
Most of files differs on shadow size and other very small visible elements, so it is possible to set similarity level, which should part of such files ignore.

## Project Requirements
- Rust compiler to compile project, nothing else

## Usage
- Install at least 2 svg libraries 
- Compile app `cargo build --release` or download prebuilt binaries(not sure if are available)
- Prepare folder with svg files to test
- Configure `settings.toml` file - `general`, `first_tool` and `other_tool` must have set all properties
- Run app e.g. via `cargo run --release`
- Open output folder to see difference - for each svg file, there should be written visible 3 files - svg, png from first app and png from other app.

Example differences that found this tool(in Japanese flag look at shadows inside red circle)

![Screenshot from 2022-08-26 16-25-58](https://user-images.githubusercontent.com/41945903/186930569-0c46657c-9054-42e0-9eb4-a539b6eccbe4.png)

## Limitations
Tool not works with non-identical images in size(cannot compare portrait image with horizontal)