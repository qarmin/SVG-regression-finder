use std::env::args;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicI32, Ordering};
use std::{fs, process};

use image_hasher::{HashAlg, HasherConfig};

use bk_tree::BKTree;
use rayon::prelude::*;
use walkdir::WalkDir;

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
    let mut files_to_check = Vec::new();
    if Path::new(&path_to_files_to_check).is_dir() {
        for entry in WalkDir::new(&path_to_files_to_check)
            .max_depth(1)
            .into_iter()
            .flatten()
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let full_path = match path.canonicalize() {
                Ok(t) => t.to_string_lossy().to_string(),
                Err(_) => continue,
            };
            if full_path.ends_with(".svg") {
                files_to_check.push(full_path);
            }
        }
    } else {
        files_to_check = match fs::read_to_string(&path_to_files_to_check) {
            Ok(t) => t
                .split('\n')
                .map(|e| e.trim())
                .map(str::to_string)
                .filter(|e| e.ends_with(".svg"))
                .collect(),
            Err(e) => {
                println!(
                    "Failed to open file {}, reason {}",
                    path_to_files_to_check, e
                );
                process::exit(1);
            }
        };
    }

    let atomic: AtomicI32 = AtomicI32::new(0);

    files_to_check.par_iter().for_each(|source_file| {
        let number = atomic.fetch_add(1, Ordering::Relaxed);
        if number % 100 == 0 && number != 0 {
            println!("-- {}/{}", number, files_to_check.len());
        }

        let thorvg_png_file;
        // let inkscape_png_file;
        let rsvg_png_file;
        // Rsvg Converter
        {
            if let Some(source) = source_file.strip_suffix(".svg") {
                rsvg_png_file = format!("{}_rsvg.png", source);
            } else {
                return;
            }
            let mut args = Vec::new();
            args.push("-o".to_string());
            args.push(rsvg_png_file.to_string());
            args.push(source_file.to_string());
            args.push("-w".to_string());
            args.push(size_of_file.to_string());
            args.push("-h".to_string()); // Workaround, which allow to check if original image was rectangle
            args.push(size_of_file.to_string());

            let _output = Command::new("rsvg-convert").args(args).output().unwrap();

            // let err = String::from_utf8(output.stderr);
            // if err.is_ok() && err != Ok("".to_string()) {
            //     let message = err.unwrap();
            //     if message.starts_with("Error reading ") {
            //         return;
            //     }
            //     println!("RSVG {:?} {}", message, source_file);
            // }
        }
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

            let err = String::from_utf8(_output.stderr);
            if let Ok(message) = err {
                if !message.is_empty() {
                    if message.starts_with("Error reading ") {
                        return;
                    }
                    println!("ThorVG {:?} {}", message, source_file);
                }
            }
        }
        // // Inkscape Converter
        // {
        //     let old_png_file;
        //     if let Some(source) = source_file.strip_suffix(".svg") {
        //         old_png_file = format!("{}.png", source);
        //         inkscape_png_file = format!("{}_inkscape.png", source);
        //     } else {
        //         return;
        //     }
        //     let args = vec![
        //         source_file.to_string(),
        //         "--export-type=png".to_string(),
        //         // "-w".to_string(),
        //         // size_of_file.to_string()),
        //         "-h".to_string(),
        //         size_of_file.to_string(),
        //     ];
        //
        //     let _output = Command::new("inkscape").args(args).output().unwrap();
        //
        //     let _ = fs::copy(&old_png_file, &inkscape_png_file);
        //     let _ = fs::remove_file(&old_png_file);
        //     // println!("Inkscape {:?}", String::from_utf8(output.stdout));
        // }

        let thorvg_image = match image::open(&thorvg_png_file) {
            Ok(t) => t,
            Err(_) => {
                // println!("Failed to open {}", thorvg_png_file);
                return;
            }
        };
        let rsvg_image = match image::open(&rsvg_png_file) {
            Ok(t) => t,
            Err(_) => {
                // println!("Failed to open {}", rsvg_png_file);
                return;
            }
        };

        // Both inkscape and rsvg works differently that ThorVG https://github.com/Samsung/thorvg/issues/1258
        if thorvg_image.width() != rsvg_image.width()
            || thorvg_image.height() != rsvg_image.height()
        {
            println!(
                "Ignored non square images thorvg {}x{}, rsvg {}x{}",
                thorvg_image.width(),
                thorvg_image.height(),
                rsvg_image.width(),
                rsvg_image.height()
            );
            return;
        }

        let hasher = HasherConfig::new()
            .hash_alg(HashAlg::Blockhash) // Looks that this is quite good hash algorithm for images with alpha, other not works well
            .hash_size(16, 16)
            .to_hasher();

        let thorvg_hash = hasher.hash_image(&thorvg_image).as_bytes().to_vec();
        let rsvg_hash = hasher.hash_image(&rsvg_image).as_bytes().to_vec();
        let mut bktree = BKTree::new(Hamming);

        bktree.add(thorvg_hash);

        let finds = bktree.find(&rsvg_hash, 9999).collect::<Vec<_>>();
        let similarity_found = match finds.get(0) {
            Some(t) => t.0,
            None => 999999,
        };

        if !finds.is_empty() && similarity_found <= similarity {
            // println!(
            //     "VALID conversion, rsvg and thorvg have same output for {}",
            //     source_file
            // );
        } else {
            // println!(
            //     "INVALID conversion, thorvg and rsvg results are different, difference {}\n\tSVG {}\n\tRsvg {}\n\tThorvg {}",
            //     similarity_found, source_file, rsvg_png_file, thorvg_png_file
            // );
            print!(
                "\tfirefox {}; firefox {}; firefox {}",
                source_file, rsvg_png_file, thorvg_png_file
            ); // I found that the best to compare images, is to open them in firefox and switch tabs,
            println!();
        }
    });
}
