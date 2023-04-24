use config::Config;
use std::collections::HashMap;

pub struct Settings {
    pub folder_with_files_to_check: String,
    pub px_size_of_generated_file: u32,
    pub ignore_conversion_step: bool,
    pub ignore_thorvg_not_supported_items: bool,
    pub similarity: u32,
    pub output_folder: String,
    pub limit_files: usize,
    pub remove_files_from_output_folder_at_start: bool,
    pub ignore_similarity_checking_step: bool,
    pub debug_show_always_output: bool,
    pub problematic_files_path: String,
    pub return_error_when_finding_invalid_files: bool,
    pub remove_problematic_files_after_copying: bool,
    // TODO timeout: u32,
    pub first_tool_name: String,
    pub first_tool_path: String,
    pub first_tool_png_name_ending: String,
    pub first_tool_arguments: String,

    pub other_tool_name: String,
    pub other_tool_path: String,
    pub other_tool_png_name_ending: String,
    pub other_tool_arguments: String,
}

pub fn load_settings() -> Settings {
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
        return_error_when_finding_invalid_files: general_settings
            ["return_error_when_finding_invalid_files"]
            .parse()
            .unwrap(),
        problematic_files_path: general_settings["problematic_files_path"].clone(),
        remove_problematic_files_after_copying: general_settings
            ["remove_problematic_files_after_copying"]
            .parse()
            .unwrap(),
        //timeout: general_settings["timeout"].parse().unwrap(),
        first_tool_name: first_tool_settings["name"].clone(),
        first_tool_path: first_tool_settings["path"].clone(),
        first_tool_png_name_ending: first_tool_settings["png_name_ending"].clone(),
        first_tool_arguments: first_tool_settings["arguments"].clone(),
        other_tool_name: other_tool_settings["name"].clone(),
        other_tool_path: other_tool_settings["path"].clone(),
        other_tool_png_name_ending: other_tool_settings["png_name_ending"].clone(),
        other_tool_arguments: other_tool_settings["arguments"].clone(),
        debug_show_always_output: general_settings["debug_show_always_output"]
            .parse()
            .unwrap(),
    }
}
