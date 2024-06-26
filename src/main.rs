#![allow(clippy::similar_names)]

use std::collections::BTreeMap;
use std::path::Path;
use std::process::exit;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::{fs, process};

use image_hasher::HashAlg;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::image_comparison::{compare_images, get_difference_between_images};
use crate::lottie::test_lottie;
use crate::setting::{load_settings, Settings};
use crate::svg_2_png::convert_svg_to_png;
use crate::thorvg::test_thorvg;

mod common;
mod image_comparison;
mod lottie;
mod setting;
mod svg_2_png;
mod thorvg;

struct Hamming;

impl bk_tree::Metric<Vec<u8>> for Hamming {
    fn distance(&self, a: &Vec<u8>, b: &Vec<u8>) -> u32 {
        hamming::distance_fast(a, b).unwrap() as u32
    }

    fn threshold_distance(&self, a: &Vec<u8>, b: &Vec<u8>, _threshold: u32) -> Option<u32> {
        Some(self.distance(a, b))
    }
}

fn find_files(settings: &Settings, extension: &str) -> Vec<String> {
    let mut files_to_check = Vec::new();
    println!(
        "Starting to collect files to check - extension {extension} in {}",
        settings.folder_with_files_to_check
    );
    if Path::new(&settings.folder_with_files_to_check).is_dir() {
        for entry in WalkDir::new(&settings.folder_with_files_to_check).max_depth(1).into_iter().flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let full_path = match path.canonicalize() {
                Ok(t) => t.to_string_lossy().to_string(),
                Err(_) => continue,
            };
            if full_path.ends_with(extension) {
                files_to_check.push(full_path);
            }
        }
    } else {
        files_to_check = match fs::read_to_string(&settings.folder_with_files_to_check) {
            Ok(t) => t.split('\n').map(str::trim).map(str::to_string).filter(|e| e.ends_with(".svg")).collect(),
            Err(e) => {
                println!("Failed to open file {}, reason {}", settings.folder_with_files_to_check, e);
                process::exit(1);
            }
        };
    }
    println!("Collected {} files to check", files_to_check.len());
    files_to_check
}

fn check_tools(settings: &Settings) {
    if settings.first_tool_path.contains('/') {
        if !Path::new(&settings.first_tool_path).is_file() {
            eprintln!("First tool not found at {}", settings.first_tool_path);
            process::exit(1);
        }
    } else {
        let path = which::which(&settings.first_tool_path);
        if path.is_err() {
            eprintln!("First tool not found at {}", settings.first_tool_path);
            process::exit(1);
        }
    }

    if settings.other_tool_path.contains('/') {
        if !Path::new(&settings.other_tool_path).is_file() {
            eprintln!("Other tool not found at {}", settings.other_tool_path);
            process::exit(1);
        }
    } else {
        let path = which::which(&settings.other_tool_path);
        if path.is_err() {
            eprintln!("Other tool not found at {}", settings.other_tool_path);
            process::exit(1);
        }
    }

    // Check if tools png output is same - should be different
    if settings.first_tool_png_name_ending == settings.other_tool_png_name_ending {
        eprintln!("First tool and other tool png name ending is same, should be different, because otherwise it will overwrite each other, and will give always same results");
        process::exit(1);
    }
}

fn test_hashers() {
    let hash_algs = [
        HashAlg::VertGradient,
        HashAlg::DoubleGradient,
        HashAlg::Blockhash,
        HashAlg::Mean,
        HashAlg::Median,
        HashAlg::Gradient,
    ];
    // let hash_algs = [HashAlg::VertGradient];

    let files = find_files(&load_settings(), ".png");
    // Remove "_rsvg" and "_thorvg" from file names
    let mut files_cleaned = files
        .iter()
        .filter(|e| e.contains("_rsvg.png") || e.contains("_thorvg.png"))
        .map(|file| file.replace("_rsvg.png", "").replace("_thorvg.png", ""))
        .collect::<Vec<_>>();
    files_cleaned.sort_unstable();
    files_cleaned.dedup();
    let dynamic_images = files_cleaned
        .into_iter()
        .map(|file| {
            let thorvg_file_name = format!("{}_thorvg.png", file);
            let rsvg_file_name = format!("{}_rsvg.png", file);
            let thorvg_image = image::open(&thorvg_file_name).unwrap();
            let rsvg_image = image::open(&rsvg_file_name).unwrap();
            (thorvg_file_name, thorvg_image, rsvg_file_name, rsvg_image)
        })
        .collect::<Vec<_>>();

    let mut differences: BTreeMap<String, u32> = BTreeMap::new();
    for (_thorvg_file_name, thorvg_image, _rsvg_file_name, rsvg_image) in dynamic_images {
        for alg in hash_algs {
            for remove_alpha in [true, false] {
                let diff = *get_difference_between_images(&[alg], &mut thorvg_image.clone(), &mut rsvg_image.clone(), remove_alpha)
                    .first()
                    .unwrap();
                let key = format!("{:?}-{}", alg, remove_alpha);
                differences.entry(key).and_modify(|e| *e += diff).or_insert(diff);
                continue;
            }
        }
    }

    for (key, value) in differences {
        println!("{}: {}", key, value);
    }
    exit(0);
}

fn main() {
    // test_hashers();

    let settings = load_settings();
    check_tools(&settings);

    // Comment to test svg files
    if settings.lottie_test {
        test_lottie(&settings);
        return;
    } else if settings.thorvg_test {
        test_thorvg(&settings);
        return;
    }

    let mut files_to_check = find_files(&settings, ".svg");
    assert!(!files_to_check.is_empty());

    if settings.limit_files != 0 {
        files_to_check = files_to_check[..settings.limit_files].to_vec();
    }
    if settings.limit_threads != 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(settings.limit_threads as usize)
            .build_global()
            .unwrap();
    }

    let atomic: AtomicI32 = AtomicI32::new(0);
    let broken_items: AtomicU32 = AtomicU32::new(0);
    let problematic_items: AtomicU32 = AtomicU32::new(0);
    let ignored_files: AtomicU32 = AtomicU32::new(0);
    // Remove output files if exists
    if settings.remove_files_from_output_folder_at_start {
        let _ = fs::remove_dir_all(&settings.output_folder);
        let _ = fs::remove_dir_all(&settings.problematic_files_path);
        let _ = fs::remove_dir_all(&settings.ignored_files_path);
    }
    let _ = fs::create_dir_all(&settings.output_folder);
    let _ = fs::create_dir_all(&settings.problematic_files_path);
    let _ = fs::create_dir_all(&settings.ignored_files_path);

    files_to_check.par_iter().for_each(|source_file| {
        let number = atomic.fetch_add(1, Ordering::Relaxed);
        if number % 100 == 0 {
            println!("-- {}/{}", number, files_to_check.len());
        }

        if settings.ignore_thorvg_not_supported_items {
            if let Ok(t) = fs::read_to_string(source_file) {
                if [
                    "<foreignObject", // https://github.com/thorvg/thorvg/issues/1255#issuecomment-2019804429
                    "<pattern", // https://github.com/thorvg/thorvg/issues/1255#issuecomment-2019804429
                    "<switch",// https://github.com/thorvg/thorvg/issues/1255#issuecomment-2019804429
                    "x:href", // https://github.com/thorvg/thorvg/issues/1255#issuecomment-2019804429
                    "mix-blend-mode:multiply", // https://github.com/thorvg/thorvg/issues/1255#issuecomment-2019804429
                    "<text",     // Fonts are not implemented
                    "<filter",   // Filters are not implemented in thorvg - but some may be implemented
                    "<image",    // Used by me build not support images, but can in future
                    "<!ENTITY", // <!ENTITY> is a xml std and it's not widely supported inside svg. https://github.com/thorvg/thorvg/issues/1255#issuecomment-1537563461
                    "<flowRoot", // this was a feature described in one of the svg tiny versions pkt 4, but was removed from the latter versions. - https://github.com/thorvg/thorvg/issues/1255#issuecomment-1537563461 -  https://www.w3.org/TR/2004/WD-SVG12-20041027/
                ]
                    .iter()
                    .any(|e| t.contains(e))
                    // Nested svg files are not supported - https://github.com/thorvg/thorvg/issues/1255#issuecomment-1537563461
                    || t.matches("<svg").count() > 1
                {
                    // println!("Ignoring {} with text", source_file);
                    ignored_files.fetch_add(1, Ordering::Relaxed);
                    let new_file_name = format!(
                        "{}/{}",
                        settings.ignored_files_path,
                        Path::new(&source_file).file_name().unwrap().to_string_lossy()
                    );
                    fs::copy(source_file, new_file_name).unwrap();
                    if settings.remove_ignored_files_after_copying {
                        let _ = fs::remove_file(source_file);
                    }
                    return;
                }
            }
        }

        let first_output_png = source_file.replace(".svg", &settings.first_tool_png_name_ending);
        let other_output_png = source_file.replace(".svg", &settings.other_tool_png_name_ending);

        if !settings.ignore_conversion_step && !convert_svg_to_png(&settings, source_file, &first_output_png, &other_output_png, &problematic_items) {
            return;
        }

        if !settings.ignore_similarity_checking_step {
            compare_images(
                source_file, &first_output_png, &other_output_png, &settings, &broken_items, &problematic_items,
            );
        }
    });

    remove_output_png_files(&settings);

    if ignored_files.load(Ordering::Relaxed) > 0 {
        println!("Ignored {} files", ignored_files.load(Ordering::Relaxed));
    }
    if broken_items.load(Ordering::Relaxed) > 0 || problematic_items.load(Ordering::Relaxed) > 0 {
        eprintln!(
            "POSSIBLE_PROBLEM - Found {} files that looks different and {} files that cannot be tested",
            broken_items.load(Ordering::Relaxed),
            problematic_items.load(Ordering::Relaxed)
        );
        if settings.return_error_when_finding_invalid_files {
            process::exit(1);
        }
    } else {
        println!("Not found any problematic files");
    }
}

fn remove_output_png_files(settings: &Settings) {
    if !settings.remove_generated_png_files_at_end {
        return;
    }

    for entry in WalkDir::new(&settings.folder_with_files_to_check).into_iter().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let full_path = match path.canonicalize() {
            Ok(t) => t.to_string_lossy().to_string(),
            Err(_) => continue,
        };
        if full_path.ends_with(".png") {
            let _ = fs::remove_file(full_path);
        }
    }
}
