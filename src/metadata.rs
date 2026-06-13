use serde::{Deserialize, Serialize};
use std::ffi::CString;

#[derive(Serialize, Deserialize)]
pub struct ContainerMetadata {
    pub state: ContainerState,
    image: String,
    args: Vec<CString>,
}

impl ContainerMetadata {
    pub fn new(image: &str, args: &[CString]) -> Self {
        Self {
            state: ContainerState::Created,
            image: image.to_string(),
            args: Vec::from(args),
        }
    }

    pub fn image(&self) -> &str {
        self.image.as_str()
    }

    pub fn args(&self) -> &[CString] {
        &self.args
    }
}

#[derive(Serialize, Deserialize)]
pub enum ContainerState {
    Created,
    Running,
    Dead,
}
