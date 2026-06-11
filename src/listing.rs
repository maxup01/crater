use crate::util::IMAGE_STORE_DIRECTORY_PATH;
use std::fs;

pub fn list_images() {
    match fs::read_dir(IMAGE_STORE_DIRECTORY_PATH) {
        Ok(entries) => {
            for entry in entries.into_iter().filter_map(|e| e.ok()) {
                println!("{:?}", entry.path());
            }
        }
        Err(e) => eprintln!("failed to read directory where images are stored: {e}"),
    }
}
