use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::common::save_problematic_file;
use crate::setting::Settings;

pub fn convert_svg_to_png(
    settings: &Settings,
    source_file: &str,
    first_output_png: &str,
    other_output_png: &str,
    problematic_items: &AtomicU32,
) -> bool {
    let possible_output_png_original = source_file.replace(".svg", ".png"); // Usually png files just are created automatically by changing extensions

    let first_command = generate_command_from_items(
        &settings.first_tool_path, &settings.first_tool_arguments, source_file, &possible_output_png_original, settings.px_size_of_generated_file,
        settings.timeout,
    );
    let other_command = generate_command_from_items(
        &settings.other_tool_path, &settings.other_tool_arguments, source_file, &possible_output_png_original, settings.px_size_of_generated_file,
        settings.timeout,
    );

    for (mut command, output_png, tool_name) in [
        (first_command, &first_output_png, &settings.first_tool_name),
        (other_command, &other_output_png, &settings.other_tool_name),
    ] {
        // Run command to convert svg to png
        let output = command
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();

        // If converted png file have same name as svg, rename it to required name
        if Path::new(&possible_output_png_original).is_file() {
            fs::copy(&possible_output_png_original, output_png)
                .unwrap_or_else(|_| panic!("Failed to copy file {possible_output_png_original} to {output_png}"));
            fs::remove_file(&possible_output_png_original).unwrap_or_else(|_| panic!("Failed to remove file {possible_output_png_original}"));
        }

        let err_message = String::from_utf8(output.stderr);
        let normal_message = String::from_utf8(output.stdout);

        if settings.debug_show_always_output {
            println!(
                "{source_file}\nERR: {err_message:?}\nOUT: {normal_message:?}\nSTATUS: {}\n",
                output.status
            );
        }
        if !output.status.success() && !settings.debug_show_always_output {
            save_problematic_file(
                &settings.problematic_files_path, tool_name, source_file, settings.remove_problematic_files_after_copying,
            );
            problematic_items.fetch_add(1, Ordering::Relaxed);
            let mut e_msg = String::new();
            if let Ok(message) = err_message {
                e_msg = message.clone();
            }
            let mut n_msg = String::new();
            if let Ok(message) = normal_message {
                n_msg = message.clone();
            }
            println!("\n\n{}\n{}\ncommand {:?} {:?}", e_msg, n_msg, command.get_program(), command.get_args());
            return false;
        }
    }
    true
}

fn generate_command_from_items(
    name: &str,
    arguments: &str,
    source_file: &str,
    output_file: &str,
    px_size_of_generated_file: u32,
    timeout: u32,
) -> Command {
    let new_arguments = arguments.replace("{SIZE}", &px_size_of_generated_file.to_string());
    let mut com = Command::new("timeout");
    com.arg("-v").arg(timeout.to_string());
    com.arg(name);
    // FILE must be renamed after splitting arguments by space, because source_file may contain spaces
    // and broke file
    com.args(
        new_arguments
            .split(' ')
            .map(|e| e.replace("{FILE}", source_file).replace("{OUTPUT_FILE}", output_file)),
    );
    com
}
