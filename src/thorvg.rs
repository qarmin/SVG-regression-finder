use crate::find_files;
use crate::setting::Settings;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::process::{exit, Command};
use std::sync::atomic::{AtomicI32, Ordering};
use walkdir::WalkDir;

pub fn test_thorvg(settings: &Settings) {
    let files_to_check = find_files(&settings, true);

    assert!(!files_to_check.is_empty());

    let _ = fs::remove_dir_all(&settings.thorvg_broken_files_path);
    let _ = fs::create_dir(&settings.thorvg_broken_files_path);

    find_broken_thorvg_files(files_to_check, &settings);
    delete_gif_files(&settings);

    exit(0);
}
fn copy_broken_file(broken_files: (String, String), settings: &Settings) {
    let (file, output) = broken_files;
    let path = Path::new(&file);
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let file_stem = path.file_stem().unwrap().to_str().unwrap();

    let _ = fs::copy(&file, format!("{}/{}", settings.thorvg_broken_files_path, file_name));
    fs::write(format!("{}/{}.txt", settings.thorvg_broken_files_path, file_stem), output).unwrap();
}
fn find_broken_thorvg_files(files_to_check: Vec<String>, settings: &Settings) {
    let atomic_counter: AtomicI32 = AtomicI32::new(0);
    let all_files = files_to_check.len();
    files_to_check.into_par_iter().for_each(|e| {
        let number = atomic_counter.fetch_add(1, Ordering::Relaxed);
        if number % 100 == 0 {
            println!("-- {}/{} - THORVG", number, all_files);
        }
        let output = Command::new(&settings.thorvg_path)
            .arg(&e)
            .args(&["-r", "200x200"])
            .output()
            .expect("Failed to execute thorvginfo");
        let all = format!("{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        let invalid_info = ["LeakSanitizer", "AddressSanitizer"];
        if invalid_info.iter().any(|e| all.contains(e)) || all.trim().len() > 200 {
            println!("Broken file {}({})\n{}\n\n", e, (all.len()), all);
            copy_broken_file((e, all), &settings);
        }
    });
}
fn delete_gif_files(settings: &Settings) {
    for entry in WalkDir::new(&settings.folder_with_files_to_check).max_depth(1).into_iter().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let full_path = match path.canonicalize() {
            Ok(t) => t.to_string_lossy().to_string(),
            Err(_) => continue,
        };
        if full_path.ends_with(".gif") {
            let _ = fs::remove_file(full_path);
        }
    }
}
