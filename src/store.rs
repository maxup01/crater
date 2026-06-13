use crate::metadata::{ContainerMetadata, ContainerState};
use error::CraterError;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub struct StoreContext;

impl StoreContext {
    const IMAGE_STORE_DIRECTORY_PATH: &str = "/crater/images";
    const CONTAINER_METADATA_STORE_PATH: &str = "/crater/metadata";

    pub fn init() -> Result<(), CraterError> {
        fs::create_dir_all(Self::IMAGE_STORE_DIRECTORY_PATH)?;
        fs::create_dir_all(Self::CONTAINER_METADATA_STORE_PATH)?;

        Ok(())
    }

    pub fn image_store_dir() -> &'static str {
        Self::IMAGE_STORE_DIRECTORY_PATH
    }

    pub fn container_metadata_store_dir() -> &'static str {
        Self::CONTAINER_METADATA_STORE_PATH
    }

    pub fn list_images() {
        match Self::list_entries(StoreContext::image_store_dir()) {
            Ok(entries) => {
                for entry in entries {
                    println!("{}", entry);
                }
            }
            Err(e) => {
                eprintln!("failed to read directory where images are stored: {e}");
            }
        }
    }

    pub fn list_containers() {
        match Self::list_entries(Self::container_metadata_store_dir()) {
            Ok(entries) => {
                for entry in entries {
                    println!("{}", entry.strip_suffix(".json").unwrap_or(&entry));
                }
            }
            Err(e) => {
                eprintln!("failed to read directory where images are stored: {e}");
            }
        }
    }

    fn list_entries<P: AsRef<Path>>(path: P) -> Result<impl Iterator<Item = String>, CraterError> {
        let entries = fs::read_dir(path)?;

        let entries = entries.into_iter().filter_map(|e| e.ok()).map(|entry| {
            let path = entry.path();

            path.file_name()
                .map(|v| v.to_string_lossy().into_owned())
                .expect("directory for crater images is corrupted")
        });

        Ok(entries)
    }
}

pub struct MetadataStore;

impl MetadataStore {
    pub fn save_container_metadata(
        name: &str,
        metadata: ContainerMetadata,
    ) -> Result<(), CraterError> {
        let metadata_json = serde_json::to_string::<ContainerMetadata>(&metadata)?;

        let metadata_file_path = Self::container_metadata_path(name);

        let mut file = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(&metadata_file_path)?;

        let _ = file.write(metadata_json.as_bytes())?;

        Ok(())
    }

    pub fn pull_container_metadata(name: &str) -> Result<ContainerMetadata, CraterError> {
        let metadata_file_path = Self::container_metadata_path(name);

        let file_content = fs::read_to_string(&metadata_file_path)?;

        let container_metadata = serde_json::from_str::<ContainerMetadata>(&file_content)?;

        Ok(container_metadata)
    }

    pub fn container_metadata_exists(name: &str) -> Result<bool, CraterError> {
        let exists = fs::exists(Self::container_metadata_path(name))?;

        Ok(exists)
    }

    pub fn update_container_state(name: &str, state: ContainerState) -> Result<(), CraterError> {
        let mut container_metadata = Self::pull_container_metadata(name)?;
        container_metadata.state = state;

        Self::save_container_metadata(name, container_metadata)
    }

    fn container_metadata_path(name: &str) -> PathBuf {
        let filename = format!("{}.json", name);
        PathBuf::from(StoreContext::container_metadata_store_dir()).join(filename)
    }
}
