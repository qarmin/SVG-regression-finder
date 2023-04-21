use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
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
    ignore_thorvg_not_supported_items: bool,
    similarity: u32,
    output_folder: String,
    limit_files: usize,
    remove_files_from_output_folder_at_start: bool,
    ignore_similarity_checking_step: bool,
    debug_show_always_output: bool,
    test_version: bool,

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
        ignore_thorvg_not_supported_items: general_settings["ignore_thorvg_not_supported_items"]
            .parse()
            .unwrap(),
        similarity: general_settings["similarity"].parse().unwrap(),
        output_folder: general_settings["output_folder"].clone(),
        limit_files: general_settings["limit_files"].parse().unwrap(),
        remove_files_from_output_folder_at_start: general_settings
            ["remove_files_from_output_folder_at_start"]
            .parse()
            .unwrap(),
        ignore_similarity_checking_step: general_settings["ignore_similarity_checking_step"]
            .parse()
            .unwrap(),
        test_version: general_settings["test_version"].parse().unwrap(),
        first_tool_path: first_tool_settings["path"].clone(),
        first_tool_png_name_ending: first_tool_settings["png_name_ending"].clone(),
        first_tool_arguments: first_tool_settings["arguments"].clone(),
        other_tool_path: other_tool_settings["path"].clone(),
        other_tool_png_name_ending: other_tool_settings["png_name_ending"].clone(),
        other_tool_arguments: other_tool_settings["arguments"].clone(),
        debug_show_always_output: general_settings["debug_show_always_output"]
            .parse()
            .unwrap(),
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
    let new_arguments = arguments.replace("{SIZE}", &px_size_of_generated_file.to_string());
    let mut comm = Command::new(name);
    // FILE must be renamed after splitting arguments by space, because source_file may contain spaces
    // and broke file
    comm.args(new_arguments.split(' ').map(|e| {
        e.replace("{FILE}", source_file)
            .replace("{OUTPUT_FILE}", output_file)
    }));
    comm
}

fn test_version(app_name: &str) {
    Command::new(app_name)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to check version of {app_name} via `{app_name} --version`, probably this is not valid path to file, if is proper and app not supports --version argument, just disable this check"))
        .wait_with_output()
        .unwrap_or_else(|_| panic!("Failed to wait into --version command"));
}

fn main() {
    let settings = load_settings();
    if settings.test_version {
        test_version(&settings.first_tool_path);
        test_version(&settings.other_tool_path);
    }

    let mut files_to_check = find_files(&settings);
    if settings.limit_files != 0 {
        files_to_check = files_to_check[..settings.limit_files].to_vec();
    }

    let atomic: AtomicI32 = AtomicI32::new(0);
    // Remove output files if exists
    if settings.remove_files_from_output_folder_at_start {
        let _ = fs::remove_dir_all(&settings.output_folder);
    }
    let _ = fs::create_dir_all(&settings.output_folder);

    files_to_check.par_iter().for_each(|source_file| {
        let number = atomic.fetch_add(1, Ordering::Relaxed);
        if number % 100 == 0 {
            println!("-- {}/{}", number, files_to_check.len());
        }

        if settings.ignore_thorvg_not_supported_items {
            if let Ok(t) = fs::read_to_string(source_file) {
                if t.contains("<text") || t.contains("<filter") || t.contains("<image")
                // https://github.com/thorvg/thorvg/issues/1367
                {
                    // println!("Ignoring {} with text", source_file);
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
                let output = command
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap()
                    .wait_with_output()
                    .unwrap();

                // Delete default created item
                if Path::new(&possible_output_png_original).is_file() {
                    let _ = fs::copy(&possible_output_png_original, output_png);
                    let _ = fs::remove_file(&possible_output_png_original);
                }

                let err_message = String::from_utf8(output.stderr);
                let normal_message = String::from_utf8(output.stdout);

                if settings.debug_show_always_output {
                    println!("{source_file}\nERR: {err_message:?}\nOUT: {normal_message:?}\nSTATUS: {}\n", output.status)
                }

                if let Ok(message) = err_message {
                    if !message.is_empty() {
                        println!(
                            "\n\n{} {} -\ncommand {:?} {:?}",
                            message,
                            source_file,
                            command.get_program(),
                            command.get_args()
                        );
                        println!("{source_file}");
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
        if !settings.ignore_similarity_checking_step {
            compare_images(source_file, &first_output_png, &other_output_png, &settings);
        }
    });
}

fn compare_images(
    source_file: &str,
    first_output_png: &str,
    other_output_png: &str,
    settings: &Settings,
) {
    let first_image = match image::open(first_output_png) {
        Ok(t) => t,
        Err(e) => {
            println!("Failed to open {first_output_png}, reason {e}");
            return;
        }
    };
    let second_image = match image::open(other_output_png) {
        Ok(t) => t,
        Err(e) => {
            println!("Failed to open {other_output_png}, reason {e}");
            return;
        }
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
        // print!("\tfirefox {source_file}; firefox {first_output_png}; firefox {other_output_png}"); // I found that the best to compare images, is to open them in firefox and switch tabs,
        copy_to_file_name(first_output_png, &settings.output_folder);
        copy_to_file_name(other_output_png, &settings.output_folder);
        copy_to_file_name(source_file, &settings.output_folder);
        // println!();
    }
}

fn copy_to_file_name(original_file: &str, output_folder: &str) {
    fs::copy(
        original_file,
        format!(
            "{}/{}",
            output_folder,
            Path::new(&original_file)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        ),
    )
    .unwrap();
}