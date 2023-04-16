use std::collections::HashMap;

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

struct Settings {
    folder_with_files_to_check: String,
    px_size_of_generated_file: u32,
    ignore_conversion_step: bool,
    ignore_with_text: bool,
    similarity: u32,
    output_folder: String,

    first_tool_path: String,
    first_tool_png_name_ending: String,
    first_tool_arguments: String,

    other_tool_path: String,
    other_tool_png_name_ending: String,
    other_tool_arguments: String,
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
    let first_tool_settings = config["first_tool"].clone();
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
        first_tool_path: first_tool_settings["path"].clone(),
        first_tool_png_name_ending: first_tool_settings["png_name_ending"].clone(),
        first_tool_arguments: first_tool_settings["arguments"].clone(),
        other_tool_path: other_tool_settings["path"].clone(),
        other_tool_png_name_ending: other_tool_settings["png_name_ending"].clone(),
        other_tool_arguments: other_tool_settings["arguments"].clone(),
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
                .map(str::trim)
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

fn generate_command_from_items(
    name: &str,
    arguments: &str,
    source_file: &str,
    output_file: &str,
    px_size_of_generated_file: u32,
) -> Command {
    let new_arguments = arguments
        .replace("{FILE}", source_file)
        .replace("{OUTPUT_FILE}", output_file)
        .replace("{SIZE}", &px_size_of_generated_file.to_string());
    let mut comm = Command::new(name);
    comm.args(new_arguments.split(' '));
    comm
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

        let first_output_png = source_file.replace(".svg", &settings.first_tool_png_name_ending);
        let other_output_png = source_file.replace(".svg", &settings.other_tool_png_name_ending);

        let possible_output_png_original = source_file.replace(".svg", ".png"); // Usually png files just are created automatically by changing extensions

        let first_command = generate_command_from_items(
            &settings.first_tool_path,
            &settings.first_tool_arguments,
            source_file,
            &possible_output_png_original,
            settings.px_size_of_generated_file,
        );
        let other_command = generate_command_from_items(
            &settings.other_tool_path,
            &settings.other_tool_arguments,
            source_file,
            &possible_output_png_original,
            settings.px_size_of_generated_file,
        );

        for (mut command, output_png) in [
            (first_command, &first_output_png),
            (other_command, &other_output_png),
        ] {
            if !settings.ignore_conversion_step {
                // Run command
                let output = command.spawn().unwrap().wait_with_output().unwrap();

                // Delete default created item
                if Path::new(&possible_output_png_original).is_file() {
                    let _ = fs::copy(&possible_output_png_original, output_png);
                    let _ = fs::remove_file(&possible_output_png_original);
                }

                let err_message = String::from_utf8(output.stderr);
                let normal_message = String::from_utf8(output.stdout);
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
        compare_images(source_file, &first_output_png, &other_output_png, &settings);
    });
}
fn compare_images(
    source_file: &str,
    first_output_png: &str,
    other_output_png: &str,
    settings: &Settings,
) {
    let Ok(first_image) = image::open(first_output_png)  else {
            println!("Failed to open {first_output_png}");
            return;

    };
    let Ok(second_image) = image::open(other_output_png) else  {
            println!("Failed to open {other_output_png}");
            return;
    };

    if second_image.width() != first_image.width() || second_image.height() != first_image.height()
    {
        println!(
            "Ignored images with non equal lengths {} {}x{}, {} {}x{}",
            other_output_png,
            second_image.width(),
            second_image.height(),
            first_output_png,
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
        None => 999_999,
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
        print!("\tfirefox {source_file}; firefox {first_output_png}; firefox {other_output_png}"); // I found that the best to compare images, is to open them in firefox and switch tabs,
        fs::copy(
            first_output_png,
            format!(
                "{}/{}",
                settings.output_folder,
                Path::new(&first_output_png)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            ),
        )
        .unwrap();
        fs::copy(
            other_output_png,
            format!(
                "{}/{}",
                settings.output_folder,
                Path::new(&other_output_png)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            ),
        )
        .unwrap();
        fs::copy(
            source_file,
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
}
