use crate::metadata::ContainerMetadata;
use error::CraterError;
use std::{fs, path::PathBuf};

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
}

pub struct MetadataStore;

impl MetadataStore {
    pub fn pull_container_metadata(name: &str) -> Result<ContainerMetadata, CraterError> {
        let filename = format!("{}.json", name);
        let metadata_file_path =
            PathBuf::from(StoreContext::container_metadata_store_dir()).join(filename);

        let file_content = fs::read_to_string(metadata_file_path)?;

        let container_metadata = serde_json::from_str::<ContainerMetadata>(&file_content)?;

        Ok(container_metadata)
    }
}
