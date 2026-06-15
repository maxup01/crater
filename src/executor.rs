use crate::{
    metadata::{ContainerMetadata, ContainerState},
    network::{ContainerSideInterface, HostSideInterface, NetworkInterface},
    store::{MetadataStore, StoreContext},
    util,
};
use nix::{
    mount::{self, MntFlags, MsFlags},
    sched::{self, CloneFlags},
    sys::wait::{self, WaitStatus},
    unistd::{self, ForkResult},
};
use std::{
    ffi::{CStr, CString},
    fs,
    path::PathBuf,
};

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

            execute(name.as_str(), metadata.image(), &args[0], args);
        });
    } else {
        let args = metadata.args();

        execute(name.as_str(), metadata.image(), &args[0], args);
    }
}

fn execute(container_name: &str, image: &str, program: &CStr, args: &[CString]) {
    let flags = CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID;

    if let Err(e) = sched::unshare(flags) {
        eprintln!("unshare failed: {e}");

        return;
    }

    let (p_read, c_write) = unistd::pipe().unwrap();
    let (c_read, p_write) = unistd::pipe().unwrap();

    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            let mut b = [0u8; 1];
            let _ = unistd::read(p_read, &mut b);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let net_if = NetworkInterface::<HostSideInterface>::new().unwrap();

                net_if
                    .create_veth_pair("veth-host", "veth-container")
                    .await
                    .unwrap();
                net_if
                    .assign_address("veth-host", "10.0.0.1".parse().unwrap(), 24)
                    .await
                    .unwrap();
                net_if.set_link_up("veth-host").await.unwrap();
                net_if
                    .move_veth_pair_end("veth-container", child.as_raw() as u32)
                    .await
                    .unwrap();
            });

            let _ = unistd::write(p_write, &[1u8]);

            if let Err(e) = util::add_process_to_cgroup(child.as_raw()) {
                eprintln!("failed to attach child process to cgroup: {e}");
            }

            match wait::waitpid(child, None) {
                Ok(WaitStatus::Exited(_, code)) => {
                    if code != 0 {
                        eprintln!("container exited with code {code}");
                    }
                }
                Ok(other) => eprintln!("container ended: {other:?}"),
                Err(e) => eprintln!("waitpid failed: {e}"),
            };

            MetadataStore::update_container_state(container_name, ContainerState::Dead)
                .unwrap_or_else(|e| {
                    eprintln!("failed to update container's metadata: {}", e);
                });
        }
        Ok(ForkResult::Child) => {
            MetadataStore::update_container_state(container_name, ContainerState::Running)
                .unwrap_or_else(|e| {
                    eprintln!("failed to update container's metadata: {}", e);
                });

            if let Err(e) = sched::unshare(CloneFlags::CLONE_NEWNET | CloneFlags::CLONE_NEWNS) {
                eprintln!("unshare failed: {e}");
            }

            let _ = unistd::write(c_write, &[1u8]);

            let mut b = [0u8; 1];
            let _ = unistd::read(c_read, &mut b);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let image_path = PathBuf::from(StoreContext::image_store_dir()).join(image);

            if let Err(e) = unistd::sethostname("container-host") {
                eprintln!("failed to set hostname for container: {}", e);
            }

            if let Err(e) = mount::mount(
                None::<&str>,
                "/",
                None::<&str>,
                MsFlags::MS_REC | MsFlags::MS_PRIVATE,
                None::<&str>,
            ) {
                eprintln!("failed to mount root directory: {}", e);
            }

            StoreContext::init_filesystem_store(container_name);

            let merged_dir = StoreContext::container_filesystem_merged(container_name);
            let old_root_temp = PathBuf::from(&merged_dir).join("oldroot");

            let mount_opts = format!(
                "lowerdir={},upperdir={},workdir={}",
                image_path.to_str().unwrap(),
                StoreContext::container_filesystem_state(container_name),
                StoreContext::container_filesystem_overlay(container_name),
            );

            if let Err(e) = mount::mount(
                Some("overlay"),
                merged_dir.as_str(),
                Some("overlay"),
                MsFlags::empty(),
                Some(mount_opts.as_str()),
            ) {
                eprintln!("failed to mount root directory: {}", e);
            }

            if let Err(e) = fs::create_dir_all(&old_root_temp) {
                eprintln!(
                    "failed to create temporary directory for previous root directory: {}",
                    e
                );
            }

            if let Err(e) = unistd::pivot_root(merged_dir.as_str(), &old_root_temp) {
                eprintln!("failed to change root directory: {}", e);
            }

            if let Err(e) = unistd::chdir("/") {
                eprintln!("failed to navigate to a new root directory: {}", e);
            }

            if let Err(e) = mount::umount2("/oldroot", MntFlags::MNT_DETACH) {
                eprintln!("failed to unmount oldroot directory: {}", e);
            }

            rt.block_on(async {
                let net_if = NetworkInterface::<ContainerSideInterface>::new().unwrap();

                net_if.set_loopback_up().await.unwrap();
                net_if
                    .assign_address("veth-container", "10.0.0.2".parse().unwrap(), 24)
                    .await
                    .unwrap();
                net_if.set_link_up("veth-container").await.unwrap();
                net_if
                    .add_default_route("10.0.0.1".parse().unwrap())
                    .await
                    .unwrap();
            });

            let _ = unistd::execvp(program, args);

            eprintln!("execvp failed");
            std::process::exit(127);
        }
        Err(e) => eprintln!("\nError: {}", e),
    }
}
