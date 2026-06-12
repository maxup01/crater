use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ContainerMetadata {
    name: String,
    state: ContainerState,
    image: String,
    args: Vec<String>,
}

impl ContainerMetadata {
    pub fn new(name: &str, image: &str, args: &[String]) -> Self {
        Self {
            name: name.to_string(),
            state: ContainerState::Created,
            image: image.to_string(),
            args: Vec::from(args),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ContainerState {
    Created,
    Running,
    Dead,
}
