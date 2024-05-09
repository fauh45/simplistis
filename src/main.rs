use core::panic;
use std::{env, fs, path::PathBuf};

use simplistis::pages::Page;

fn main() {
    let args: Vec<String> = env::args().collect();

    let Some(template_directory_raw) = args.get(1) else {
        println!("No template directory root given, usage: simplistis [template directory root] [output directory root]");
        panic!()
    };
    let Some(output_directory_root) = args.get(2) else {
        println!("No template directory root given, usage: simplistis [template directory root] [output directory root]");
        panic!()
    };

    let template_dir = PathBuf::from(template_directory_raw);
    let output_dir = PathBuf::from(output_directory_root);

    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).unwrap();
    }

    let pages = Page::from_dir(&template_dir).unwrap();
    pages.render_all(&output_dir).unwrap();
}
