use std::collections::HashMap;

use config::Config;

pub struct Settings {
    pub folder_with_files_to_check: String,
    pub ignored_files_path: String,
    pub px_size_of_generated_file: u32,
    pub ignore_conversion_step: bool,
    pub ignore_thorvg_not_supported_items: bool,
    pub similarity: u32,
    pub output_folder: String,
    pub limit_threads: u32,
    pub limit_files: usize,
    pub remove_files_from_output_folder_at_start: bool,
    pub ignore_similarity_checking_step: bool,
    pub debug_show_always_output: bool,
    pub problematic_files_path: String,
    pub return_error_when_finding_invalid_files: bool,
    pub remove_problematic_files_after_copying: bool,
    pub remove_broken_files_after_copying: bool,
    pub remove_generated_png_files_at_end: bool,
    pub remove_ignored_files_after_copying: bool,
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
    let settings = Config::builder().add_source(config::File::with_name("settings")).build().unwrap();
    let config = settings.try_deserialize::<HashMap<String, HashMap<String, String>>>().unwrap();

    let gs = config["general"].clone();
    let fts = config["first_tool"].clone();
    let ots = config["other_tool"].clone();
    Settings {
        folder_with_files_to_check: gs["folder_with_files_to_check"].clone(),
        ignored_files_path: gs["ignored_files_path"].clone(),
        px_size_of_generated_file: gs["px_size_of_generated_file"].parse().unwrap(),
        ignore_conversion_step: gs["ignore_conversion_step"].parse().unwrap(),
        ignore_thorvg_not_supported_items: gs["ignore_thorvg_not_supported_items"].parse().unwrap(),
        similarity: gs["similarity"].parse().unwrap(),
        output_folder: gs["output_folder"].clone(),
        limit_files: gs["limit_files"].parse().unwrap(),
        limit_threads: gs["limit_threads"].parse().unwrap(),
        remove_files_from_output_folder_at_start: gs["remove_files_from_output_folder_at_start"].parse().unwrap(),
        ignore_similarity_checking_step: gs["ignore_similarity_checking_step"].parse().unwrap(),
        problematic_files_path: gs["problematic_files_path"].clone(),
        return_error_when_finding_invalid_files: gs["return_error_when_finding_invalid_files"].parse().unwrap(),
        remove_problematic_files_after_copying: gs["remove_problematic_files_after_copying"].parse().unwrap(),
        remove_broken_files_after_copying: gs["remove_broken_files_after_copying"].parse().unwrap(),
        remove_generated_png_files_at_end: gs["remove_generated_png_files_at_end"].parse().unwrap(),
        remove_ignored_files_after_copying: gs["remove_ignored_files_after_copying"].parse().unwrap(),

        //timeout: gs["timeout"].parse().unwrap(),
        first_tool_name: fts["name"].clone(),
        first_tool_path: fts["path"].clone(),
        first_tool_png_name_ending: fts["png_name_ending"].clone(),
        first_tool_arguments: fts["arguments"].clone(),
        other_tool_name: ots["name"].clone(),
        other_tool_path: ots["path"].clone(),
        other_tool_png_name_ending: ots["png_name_ending"].clone(),
        other_tool_arguments: ots["arguments"].clone(),
        debug_show_always_output: gs["debug_show_always_output"].parse().unwrap(),
    }
}
