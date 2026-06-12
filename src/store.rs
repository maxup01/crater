use error::CraterError;
use std::fs;

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
}
