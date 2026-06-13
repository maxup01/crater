use crate::store::StoreContext;
use std::fs;

pub fn list_images() {
    match fs::read_dir(StoreContext::image_store_dir()) {
        Ok(entries) => {
            for entry in entries.into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();

                let file_name = path
                    .file_name()
                    .map(|v| v.to_string_lossy())
                    .expect("directory for crater images is corrupted");

                println!("{}", file_name);
            }
        }
        Err(e) => eprintln!("failed to read directory where images are stored: {e}"),
    }
}
