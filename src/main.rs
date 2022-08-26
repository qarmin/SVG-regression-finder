use std::env::args;
use std::process::Command;
use std::{fs, process};

use image_hasher::{HashAlg, HasherConfig};

use bk_tree::BKTree;
use rayon::prelude::*;

struct Hamming;

impl bk_tree::Metric<Vec<u8>> for Hamming {
    fn distance(&self, a: &Vec<u8>, b: &Vec<u8>) -> u32 {
        hamming::distance_fast(a, b).unwrap() as u32
    }

    fn threshold_distance(&self, a: &Vec<u8>, b: &Vec<u8>, _threshold: u32) -> Option<u32> {
        Some(self.distance(a, b))
    }
}

fn main() {
    let thorvg_path;
    let path_to_files_to_check;
    let size_of_file;
    let similarity;

    let os_args: Vec<_> = args().collect();
    if os_args.len() >= 5 {
        thorvg_path = os_args[1].clone();
        path_to_files_to_check = os_args[2].clone();
        size_of_file = os_args[3].parse::<u32>().unwrap();
        similarity = os_args[4].parse::<u32>().unwrap();
    } else {
        println!("You need to set 4 arguments - thorvg path, file with svg files to check(one per line), size of file(width is same as height), similarity(0 means very similar, bigger values check for less similar svgs)");
        process::exit(1);
    }

    let lines: Vec<_> = match fs::read_to_string(&path_to_files_to_check) {
        Ok(t) => t.split('\n').map(str::to_string).collect(),
        Err(e) => {
            println!(
                "Failed to open file {}, reason {}",
                path_to_files_to_check, e
            );
            process::exit(1);
        }
    };

    lines.par_iter().for_each(|line| {
        let line = line.trim();
        if !line.ends_with(".svg") {
            return;
        }
        let source_file = line;

        let thorvg_png_file;
        let inkscape_png_file;
        // ThorVG Converter
        {
            let old_png_file;
            if let Some(source) = source_file.strip_suffix(".svg") {
                old_png_file = format!("{}.png", source);
                thorvg_png_file = format!("{}_thorvg.png", source);
            } else {
                return;
            }
            let mut args = Vec::new();
            args.push(source_file.to_string());
            args.push("-r".to_string());
            args.push(format!("{}x{}", size_of_file, size_of_file));

            let _output = Command::new(&thorvg_path).args(args).output().unwrap();

            let _ = fs::copy(&old_png_file, &thorvg_png_file);
            let _ = fs::remove_file(&old_png_file);
            // println!("ThorVG {:?}", String::from_utf8(output.stdout));
        }
        // Inkscape Converter
        {
            let old_png_file;
            if let Some(source) = source_file.strip_suffix(".svg") {
                old_png_file = format!("{}.png", source);
                inkscape_png_file = format!("{}_inkscape.png", source);
            } else {
                return;
            }
            let args = vec![
                source_file.to_string(),
                "--export-type=png".to_string(),
                // "-w".to_string(),
                // size_of_file.to_string()),
                "-h".to_string(),
                size_of_file.to_string(),
            ];

            let _output = Command::new("inkscape").args(args).output().unwrap();

            let _ = fs::copy(&old_png_file, &inkscape_png_file);
            let _ = fs::remove_file(&old_png_file);
            // println!("Inkscape {:?}", String::from_utf8(output.stdout));
        }

        let thorvg_image = match image::open(&thorvg_png_file) {
            Ok(t) => t,
            Err(_) => {
                // println!("Failed to open {}", thorvg_png_file);
                return;
            }
        };
        let inkscape_image = match image::open(&inkscape_png_file) {
            Ok(t) => t,
            Err(_) => {
                // println!("Failed to open {}", inkscape_png_file);
                return;
            }
        };

        if thorvg_image.width() != inkscape_image.width()
            || thorvg_image.height() != inkscape_image.height()
        {
            // println!("Ignored non square images thorvg {}x{}, inkscape {}x{}", thorvg_image.width(),thorvg_image.height(), inkscape_image.width(),inkscape_image.height());
            return;
        }

        let hasher = HasherConfig::new()
            .hash_alg(HashAlg::Blockhash) // Looks that this is quite good hash algorithm for images with alpha, other not works well
            .hash_size(16, 16)
            .to_hasher();

        let thorvg_hash = hasher.hash_image(&thorvg_image).as_bytes().to_vec();
        let inkscape_hash = hasher.hash_image(&inkscape_image).as_bytes().to_vec();
        let mut bktree = BKTree::new(Hamming);

        bktree.add(thorvg_hash);

        let finds = bktree.find(&inkscape_hash, 9999).collect::<Vec<_>>();
        let similarity_found = match finds.get(0) {
            Some(t) => t.0,
            None => 999999,
        };

        if !finds.is_empty() && similarity_found <= similarity {
            // println!(
            //     "VALID conversion, inkscape and thorvg have same output for {}",
            //     source_file
            // );
        } else {
            // println!(
            //     "INVALID conversion, thorvg and inkscape results are different, difference {}\n\tSVG {}\n\tInkscape {}\n\tThorvg {}",
            //     similarity_found, source_file, inkscape_png_file, thorvg_png_file
            // );
            print!(
                "\tfirefox {}; firefox {}; firefox {}",
                source_file, inkscape_png_file, thorvg_png_file
            ); // I found that the best to compare images, is to open them in firefox and switch tabs,
            println!();
        }
    });
}
