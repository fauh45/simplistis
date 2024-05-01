use std::{
    fs,
    path::{Path, PathBuf},
};

fn get_clean_list_of_contents<P: AsRef<Path>>(content_root_dir: &P) -> Vec<PathBuf> {
    let mut list_of_contents = Vec::<PathBuf>::new();

    if let Ok(entries) = fs::read_dir(content_root_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() && entry.path().ends_with(".md") {
                    list_of_contents.push(entry.path());
                }
            }
        }
    }

    list_of_contents
}
