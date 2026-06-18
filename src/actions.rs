use crate::executor;
use crate::{
    metadata::{ContainerMetadata, ContainerState},
    store::{MetadataStore, StoreContext},
};
use nix::unistd::{self, ForkResult};
use std::ffi::CString;

pub fn create_container(name: String, image: String, args: Vec<CString>) {
    let exists = StoreContext::container_metadata_exists(name.as_str()).unwrap_or_else(|e| {
        eprintln!("failed to check if container metadata file exists: {}", e);

        std::process::exit(1);
    });

    if exists {
        eprintln!("container with name {} already exists", name);

        std::process::exit(1);
    }

    let container_metadata = ContainerMetadata::new(image.as_str(), &args);

    if let Err(e) = MetadataStore::save_container_metadata(name.as_str(), container_metadata) {
        eprintln!("failed to create container: {}", e);

        std::process::exit(1);
    }
}

pub fn run_container(name: String, detach: bool) {
    let metadata = MetadataStore::pull_container_metadata(name.as_str()).unwrap_or_else(|e| {
        eprintln!("failed to retrieve container's metadata: {}", e);

        std::process::exit(1);
    });

    if matches!(metadata.state, ContainerState::Running) {
        eprintln!("container is already running");

        std::process::exit(0);
    }

    if detach {
        detach_process(move || {
            let args = metadata.args();

            executor::execute(name.as_str(), metadata.image(), &args[0], args);
        });
    } else {
        let args = metadata.args();

        executor::execute(name.as_str(), metadata.image(), &args[0], args);
    }
}

fn detach_process<F: FnOnce()>(action: F) {
    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { .. }) => {}
        Ok(ForkResult::Child) => {
            if let Err(e) = unistd::setsid() {
                eprintln!("process detaching failed: {e}");

                std::process::exit(1);
            }

            action();
        }
        Err(e) => eprintln!("\nError: {}", e),
    }
}
