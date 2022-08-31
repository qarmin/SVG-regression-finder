use std::env::args;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicI32, Ordering};
use std::{fs, process};

use bk_tree::BKTree;
use image_hasher::{HashAlg, HasherConfig};
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

struct BasicInfo {
    name: String,
    command: String,
    output_png_added: &'static str,
    output_png: String,
    arguments: Vec<String>,
    possible_output_png_original: String,
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
        if number % 100 == 0 {
            println!("-- {}/{}", number, files_to_check.len());
        }

        let mut fields: Vec<_> = vec![
            // BasicInfo {
            //     name: "Rsvg".to_string(),
            //     command: "rsvg-convert".to_string(),
            //     output_png_added: "_rsvg.png",
            //     possible_output_png_original: "".to_string(),
            //     output_png: "".to_string(),
            //     arguments: vec![
            //         source_file.to_string(),
            //         "-o".to_string(),
            //         "OUTPUT_FILE".to_string(),
            //         "-w".to_string(),
            //         size_of_file.to_string(),
            //         "-h".to_string(),
            //         size_of_file.to_string(),
            //     ],
            // },
            BasicInfo {
                name: "Thorvg".to_string(),
                command: thorvg_path.clone(),
                output_png_added: "_thorvg.png",
                possible_output_png_original: "".to_string(),
                output_png: "".to_string(),
                arguments: vec![
                    source_file.to_string(),
                    "-r".to_string(),
                    format!("{}x{}", size_of_file, size_of_file),
                ],
            },
            BasicInfo {
                name: "Thorvg PR".to_string(),
                command: "/home/rafal/test/mg/build/src/bin/svg2png/svg2png".to_string(),
                output_png_added: "_thorvg_PR.png",
                possible_output_png_original: "".to_string(),
                output_png: "".to_string(),
                arguments: vec![
                    source_file.to_string(),
                    "-r".to_string(),
                    format!("{}x{}", size_of_file, size_of_file),
                ],
            },
            // BasicInfo {
            //     name: "Inkscape".to_string(),
            //     command: "inkscape".to_string(),
            //     output_png_added: "_inkscape.png",
            //     possible_output_png_original: "".to_string(),
            //     output_png: "".to_string(),
            //     arguments: vec![
            //         source_file.to_string(),
            //         "--export-type=png".to_string(),
            //         "-w".to_string(),
            //         size_of_file.to_string(),
            //         "-h".to_string(),
            //         size_of_file.to_string(),
            //     ],
            // },
        ];
        assert_eq!(fields.len(), 2); // Only 2 are supported

        for field in fields.iter_mut() {
            field.output_png = source_file.replace(".svg", field.output_png_added);
            field.possible_output_png_original = source_file.replace(".svg", ".png");

            // Prepare arguments
            let mut args = Vec::new();
            for argument in &field.arguments {
                args.push(argument.replace("OUTPUT_FILE", &field.output_png))
            }

            // Run command
            let _output = Command::new(&field.command).args(args).output().unwrap();

            // Delete default created item
            if Path::new(&field.possible_output_png_original).is_file() {
                let _ = fs::copy(&field.possible_output_png_original, &field.output_png);
                let _ = fs::remove_file(&field.possible_output_png_original);
            }

            let err_message = String::from_utf8(_output.stderr);
            let normal_message = String::from_utf8(_output.stdout);
            if let Ok(message) = err_message {
                if !message.is_empty() {
                    println!("{} {:?} {}", field.name, message, source_file);
                }
            }
            if let Ok(message) = normal_message {
                if !message.is_empty() {
                    // println!("{} {:?} {}", field.name, message, source_file);
                }
            }
        }

        let second_image = match image::open(&fields[1].output_png) {
            Ok(t) => t,
            Err(_) => {
                println!("Failed to open {}", fields[1].output_png);
                return;
            }
        };
        let first_image = match image::open(&fields[0].output_png) {
            Ok(t) => t,
            Err(_) => {
                println!("Failed to open {}", fields[0].output_png);
                return;
            }
        };

        // Both inkscape and rsvg works differently that ThorVG https://github.com/Samsung/thorvg/issues/1258
        if second_image.width() != first_image.width()
            || second_image.height() != first_image.height()
        {
            println!(
                "Ignored non square images thorvg {}x{}, rsvg {}x{}",
                second_image.width(),
                second_image.height(),
                first_image.width(),
                first_image.height()
            );
            return;
        }

        let hasher = HasherConfig::new()
            .hash_alg(HashAlg::Blockhash) // Looks that this is quite good hash algorithm for images with alpha, other not works well
            .hash_size(16, 16)
            .to_hasher();

        let second_image_hash = hasher.hash_image(&second_image).as_bytes().to_vec();
        let first_image_hash = hasher.hash_image(&first_image).as_bytes().to_vec();
        let mut bktree = BKTree::new(Hamming);

        bktree.add(second_image_hash);

        let finds = bktree.find(&first_image_hash, 9999).collect::<Vec<_>>();
        let similarity_found = match finds.get(0) {
            Some(t) => t.0,
            None => 999999,
        };

        if !finds.is_empty() && similarity_found <= similarity {
            // println!(
            //     "VALID conversion, {} and {} have same output for {}",
            //     fields[0].name, fields[1].name, source_file
            // );
        } else {
            // println!(
            //     "INVALID conversion, {} and {} results are different, difference {}\n\tSVG {}\n\tFirst {}\n\tSecond {}",
            //     fields[0].name,fields[1].name,similarity_found, source_file, fields[0].name,fields[1].name
            // );
            print!(
                "\tfirefox {}; firefox {}; firefox {}",
                source_file, fields[0].output_png, fields[1].output_png
            ); // I found that the best to compare images, is to open them in firefox and switch tabs,
            println!();
        }
    });
}
