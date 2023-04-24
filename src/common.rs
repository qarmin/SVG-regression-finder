use std::fs;
use std::path::Path;

pub fn save_problematic_file(problematic_files_path: &str, svg_tool_name: &str, broken_svg_path: &str, remove_problematic_files_after_copying: bool) {
    let file_name = Path::new(broken_svg_path).file_name().unwrap().to_str().unwrap().to_string();
    let new_path = format!("{problematic_files_path}/{svg_tool_name}");
    let new_file_path = format!("{new_path}/{file_name}");

    if remove_problematic_files_after_copying {
        let _ = fs::copy(broken_svg_path, new_file_path);
        let _ = fs::remove_file(broken_svg_path);
    } else {
        fs::copy(broken_svg_path, new_file_path).unwrap();
    }
}
