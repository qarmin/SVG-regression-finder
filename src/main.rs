use std::collections::HashMap;
use std::env::args;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicI32, Ordering};
use std::{fs, process};

use bk_tree::BKTree;
use config::Config;
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

struct Settings {
    folder_with_files_to_check: String,
    px_size_of_generated_file: u32,
    ignore_conversion_step: bool,
    ignore_with_text: bool,
    similarity: u32,
    output_folder: String,

    thorvg_path: String,
    other_tool_path: String,
}

fn load_settings() -> Settings {
    let settings = Config::builder()
        .add_source(config::File::with_name("settings"))
        .build()
        .unwrap();
    let config = settings
        .try_deserialize::<HashMap<String, HashMap<String, String>>>()
        .unwrap();

    let general_settings = config["general"].clone();
    let thorvg_settings = config["thorvg"].clone();
    let other_tool_settings = config["other_tool"].clone();
    Settings {
        folder_with_files_to_check: general_settings["folder_with_files_to_check"].clone(),
        px_size_of_generated_file: general_settings["px_size_of_generated_file"]
            .parse()
            .unwrap(),
        ignore_conversion_step: general_settings["ignore_conversion_step"].parse().unwrap(),
        ignore_with_text: general_settings["ignore_with_text"].parse().unwrap(),
        similarity: general_settings["similarity"].parse().unwrap(),
        output_folder: general_settings["output_folder"].clone(),
        thorvg_path: thorvg_settings["path"].clone(),
        other_tool_path: other_tool_settings["path"].clone(),
    }
}

fn find_files(settings: &Settings) -> Vec<String> {
    let mut files_to_check = Vec::new();
    if Path::new(&settings.folder_with_files_to_check).is_dir() {
        for entry in WalkDir::new(&settings.folder_with_files_to_check)
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
        files_to_check = match fs::read_to_string(&settings.folder_with_files_to_check) {
            Ok(t) => t
                .split('\n')
                .map(|e| e.trim())
                .map(str::to_string)
                .filter(|e| e.ends_with(".svg"))
                .collect(),
            Err(e) => {
                println!(
                    "Failed to open file {}, reason {}",
                    settings.folder_with_files_to_check, e
                );
                process::exit(1);
            }
        };
    }
    files_to_check
}

fn main() {
    let settings = load_settings();
    let files_to_check = find_files(&settings);
    let files_to_check = files_to_check[..100].to_vec(); // TODO remove it
    let atomic: AtomicI32 = AtomicI32::new(0);

    files_to_check.par_iter().for_each(|source_file| {
        let number = atomic.fetch_add(1, Ordering::Relaxed);
        if number % 100 == 0 {
            println!("-- {}/{}", number, files_to_check.len());
        }

        let mut fields: Vec<_> = vec![
            BasicInfo {
                name: "Rsvg".to_string(),
                command: "rsvg-convert".to_string(),
                output_png_added: "_rsvg.png",
                possible_output_png_original: "".to_string(),
                output_png: "".to_string(),
                arguments: vec![
                    source_file.to_string(),
                    "-o".to_string(),
                    "OUTPUT_FILE".to_string(),
                    "-w".to_string(),
                    settings.px_size_of_generated_file.to_string(),
                    "-h".to_string(),
                    settings.px_size_of_generated_file.to_string(),
                ],
            },
            // BasicInfo {
            //     name: "Resvg".to_string(),
            //     command: "resvg".to_string(),
            //     output_png_added: "_resvg.png",
            //     possible_output_png_original: "".to_string(),
            //     output_png: "".to_string(),
            //     arguments: vec![
            //         source_file.to_string(),
            //         "OUTPUT_FILE".to_string(),
            //         "-w".to_string(),
            //         size_of_file.to_string(),
            //         "-h".to_string(),
            //         size_of_file.to_string(),
            //     ],
            // },
            BasicInfo {
                name: "Thorvg".to_string(),
                command: settings.thorvg_path.clone(),
                output_png_added: "_thorvg.png",
                possible_output_png_original: "".to_string(),
                output_png: "".to_string(),
                arguments: vec![
                    source_file.to_string(),
                    "-r".to_string(),
                    format!(
                        "{}x{}",
                        settings.px_size_of_generated_file, settings.px_size_of_generated_file
                    ),
                ],
            },
            // BasicInfo {
            //     name: "Thorvg PR".to_string(),
            //     command: "/home/rafal/test/mg/build/src/bin/svg2png/svg2png".to_string(),
            //     output_png_added: "_thorvg_PR.png",
            //     possible_output_png_original: "".to_string(),
            //     output_png: "".to_string(),
            //     arguments: vec![
            //         source_file.to_string(),
            //         "-r".to_string(),
            //         format!("{}x{}", size_of_file, size_of_file),
            //     ],
            // },
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

        if settings.ignore_with_text {
            match fs::read_to_string(source_file) {
                Ok(t) => {
                    if t.contains("</text>") || t.contains("</filter>") {
                        // println!("Ignoring {} with text", source_file);
                        return;
                    }
                }
                Err(_) => {
                    return;
                }
            }
        }

        for field in fields.iter_mut() {
            field.output_png = source_file.replace(".svg", field.output_png_added);
            field.possible_output_png_original = source_file.replace(".svg", ".png");

            // Prepare arguments
            let mut args = Vec::new();
            for argument in &field.arguments {
                args.push(argument.replace("OUTPUT_FILE", &field.output_png))
            }

            if !settings.ignore_conversion_step {
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
                        // println!("{} {:?} {}", field.name, message, source_file);
                        return;
                    }
                }
                if let Ok(message) = normal_message {
                    if !message.is_empty() {
                        // println!("{} {:?} {}", field.name, message, source_file);
                    }
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

        if second_image.width() != first_image.width()
            || second_image.height() != first_image.height()
        {
            println!(
                "Ignored non square images {} {}x{}, {} {}x{}",
                fields[1].output_png,
                second_image.width(),
                second_image.height(),
                fields[0].output_png,
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

        if !finds.is_empty() && similarity_found <= settings.similarity {
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
            fs::copy(
                &fields[0].output_png,
                format!(
                    "{}/{}",
                    settings.output_folder,
                    Path::new(&fields[0].output_png)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                ),
            )
            .unwrap();
            fs::copy(
                &fields[1].output_png,
                format!(
                    "{}/{}",
                    settings.output_folder,
                    Path::new(&fields[1].output_png)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                ),
            )
            .unwrap();
            fs::copy(
                &source_file,
                format!(
                    "{}/{}",
                    settings.output_folder,
                    Path::new(&source_file)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                ),
            )
            .unwrap();
            println!();
        }
    });
}
