use crate::metadata::ContainerMetadata;
use error::CraterError;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
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

    fn container_metadata_path(name: &str) -> PathBuf {
        let filename = format!("{}.json", name);
        PathBuf::from(StoreContext::container_metadata_store_dir()).join(filename)
    }
}
