use std::cmp::{max, min};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use bk_tree::BKTree;
use image::{DynamicImage, GenericImage, GenericImageView};
use image_hasher::{HashAlg, HasherConfig};

use crate::common::save_problematic_file;
use crate::setting::Settings;
use crate::Hamming;

pub fn remove_alpha_channel(dynamic_image: &mut DynamicImage) {
    let height = dynamic_image.height();
    let width = dynamic_image.width();
    for y in 0..height {
        for x in 0..width {
            let mut px = dynamic_image.get_pixel(x, y);
            // TODO current solution not works for fully transparent SVG
            // Looks that different tools differently recognizes alpha, so for now
            // Everything that contains alpha is changed to totally white pixel, which should help
            // To remove a lot of false positives(I expect very few false negatives)
            if px.0[3] != 255 {
                // px.0[0] = 0;
                // px.0[1] = 0;
                // px.0[2] = 0;
                px.0[3] = 255;
                dynamic_image.put_pixel(x, y, px);
            }
        }
    }
    // Save alpha
    // let path = format!("/home/rafal/Desktop/Untitled Folder 5/{}.png", thread_rng().gen::<u32>());
    // dynamic_image.save(path).unwrap();
}

pub fn compare_images(
    source_file: &str,
    first_output_png: &str,
    other_output_png: &str,
    settings: &Settings,
    broken_items: &AtomicU32,
    problematic_items: &AtomicU32,
) {
    let mut first_image = match image::open(first_output_png) {
        Ok(t) => t,
        Err(e) => {
            save_problematic_file(
                &settings.problematic_files_path, &settings.first_tool_name, source_file, settings.remove_problematic_files_after_copying,
            );
            println!("Failed to open {first_output_png}, reason {e} (Probably app failed to generate png file)");
            problematic_items.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };
    let mut second_image = match image::open(other_output_png) {
        Ok(t) => t,
        Err(e) => {
            save_problematic_file(
                &settings.problematic_files_path, &settings.other_tool_name, source_file, settings.remove_problematic_files_after_copying,
            );
            println!("Failed to open {other_output_png}, reason {e}");
            problematic_items.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };

    if second_image.width() != first_image.width() || second_image.height() != first_image.height() {
        save_problematic_file(
            &settings.problematic_files_path, &settings.first_tool_name, source_file, settings.remove_problematic_files_after_copying,
        );
        println!(
            "Ignored images with non equal lengths {} {}x{}, {} {}x{} - diff {}x{}",
            other_output_png,
            second_image.width(),
            second_image.height(),
            first_output_png,
            first_image.width(),
            first_image.height(),
            max(second_image.width(), first_image.width()) - min(second_image.width(), first_image.width()),
            max(second_image.height(), first_image.height()) - min(second_image.height(), first_image.height()),
        );
        problematic_items.fetch_add(1, Ordering::Relaxed);
        return;
    }

    let difference_between = get_difference_between_images(&[HashAlg::Median, HashAlg::Mean], &mut first_image, &mut second_image, true);

    if difference_between.iter().any(|e| e <= &settings.max_difference) {
    } else {
        copy_to_file_name(first_output_png, &settings.output_folder);
        copy_to_file_name(other_output_png, &settings.output_folder);
        copy_to_file_name(source_file, &settings.output_folder);
        broken_items.fetch_add(1, Ordering::Relaxed);
        if settings.remove_broken_files_after_copying {
            fs::remove_file(source_file).unwrap();
        }
    }
}

pub fn get_difference_between_images(
    hash_algs: &[HashAlg],
    first_image: &mut DynamicImage,
    second_image: &mut DynamicImage,
    remove_alpha: bool,
) -> Vec<u32> {
    if remove_alpha {
        remove_alpha_channel(first_image);
        remove_alpha_channel(second_image);
    }
    let mut differences = vec![];
    for hash_alg in hash_algs {
        let hasher = HasherConfig::new().hash_alg(*hash_alg).hash_size(16, 16).to_hasher(); // 8 // 17

        let second_image_hash = hasher.hash_image(second_image).as_bytes().to_vec();
        let first_image_hash = hasher.hash_image(first_image).as_bytes().to_vec();
        let mut bktree = BKTree::new(Hamming);

        bktree.add(second_image_hash);

        let finds = bktree.find(&first_image_hash, 9999).collect::<Vec<_>>();
        let difference_between = match finds.first() {
            Some(t) => t.0,
            None => 999_999,
        };
        differences.push(difference_between);
    }
    differences
}

pub fn copy_to_file_name(original_file: &str, output_folder: &str) {
    fs::copy(
        original_file,
        format!("{}/{}", output_folder, Path::new(&original_file).file_name().unwrap().to_str().unwrap()),
    )
    .unwrap();
}
