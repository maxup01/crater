use crate::{
    metadata::ContainerState,
    network::{Bridge, ContainerSideInterface, HostSideInterface, NetworkInterface},
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

pub fn execute(container_name: &str, image: &str, program: &CStr, args: &[CString]) {
    let flags = CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID;

    if let Err(e) = sched::unshare(flags) {
        eprintln!("unshare failed: {e}");

        std::process::exit(1);
    }

    let host_side_veth_name = format!("veth-{}-h", container_name);
    let container_side_veth_name = format!("veth-{}-c", container_name);

    let (p_read, c_write) = unistd::pipe().unwrap_or_else(|e| {
        eprintln!("failed to create pipe: {}", e);

        std::process::exit(1);
    });

    let (c_read, p_write) = unistd::pipe().unwrap_or_else(|e| {
        eprintln!("failed to create pipe: {}", e);

        std::process::exit(1);
    });

    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            let mut b = [0u8; 1];
            let _ = unistd::read(p_read, &mut b);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap_or_else(|e| {
                    eprintln!("failed to create tokio runtime: {}", e);

                    std::process::exit(1);
                });

            rt.block_on(async {
                Bridge::create().await.unwrap();

                let net_if = NetworkInterface::<HostSideInterface>::new().unwrap_or_else(|e| {
                    eprintln!("failed to create host side network interface: {}", e);

                    std::process::exit(1);
                });

                net_if
                    .create_veth_pair(&host_side_veth_name, &container_side_veth_name)
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("failed to create veth pair: {}", e);

                        std::process::exit(1);
                    });
                net_if
                    .set_link_up(&host_side_veth_name)
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("failed to link veth for container: {}", e);

                        std::process::exit(1);
                    });
                net_if
                    .move_veth_pair_end(&container_side_veth_name, child.as_raw() as u32)
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("failed to move veth pair end to container: {}", e);

                        std::process::exit(1);
                    });
                net_if
                    .link_to_bridge(&host_side_veth_name)
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("failed to link bridge with container: {}", e);

                        std::process::exit(1);
                    });
            });

            let _ = unistd::write(p_write, &[1u8]);

            if let Err(e) = util::add_process_to_cgroup(child.as_raw()) {
                eprintln!("failed to attach child process to cgroup: {e}");

                std::process::exit(1);
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

                    std::process::exit(1);
                });
        }
        Ok(ForkResult::Child) => {
            MetadataStore::update_container_state(container_name, ContainerState::Running)
                .unwrap_or_else(|e| {
                    eprintln!("failed to update container's metadata: {}", e);

                    std::process::exit(1);
                });

            if let Err(e) = sched::unshare(CloneFlags::CLONE_NEWNET | CloneFlags::CLONE_NEWNS) {
                eprintln!("unshare failed: {e}");

                std::process::exit(1);
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

                std::process::exit(1);
            }

            if let Err(e) = mount::mount(
                None::<&str>,
                "/",
                None::<&str>,
                MsFlags::MS_REC | MsFlags::MS_PRIVATE,
                None::<&str>,
            ) {
                eprintln!("failed to mount root directory: {}", e);

                std::process::exit(1);
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

                std::process::exit(1);
            }

            if let Err(e) = fs::create_dir_all(&old_root_temp) {
                eprintln!(
                    "failed to create temporary directory for previous root directory: {}",
                    e
                );

                std::process::exit(1);
            }

            if let Err(e) = unistd::pivot_root(merged_dir.as_str(), &old_root_temp) {
                eprintln!("failed to change root directory: {}", e);

                std::process::exit(1);
            }

            if let Err(e) = unistd::chdir("/") {
                eprintln!("failed to navigate to a new root directory: {}", e);

                std::process::exit(1);
            }

            if let Err(e) = mount::umount2("/oldroot", MntFlags::MNT_DETACH) {
                eprintln!("failed to unmount oldroot directory: {}", e);

                std::process::exit(1);
            }

            rt.block_on(async {
                let net_if =
                    NetworkInterface::<ContainerSideInterface>::new().unwrap_or_else(|e| {
                        eprintln!("failed to create container's network interface: {}", e);

                        std::process::exit(1);
                    });

                net_if.set_loopback_up().await.unwrap_or_else(|e| {
                    eprintln!("failed to set up container's loopback device: {}", e);

                    std::process::exit(1);
                });
                net_if
                    .assign_address(&container_side_veth_name, "10.0.0.2".parse().unwrap(), 24)
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("failed to set up container's loopback device: {}", e);

                        std::process::exit(1);
                    });
                net_if
                    .set_link_up(&container_side_veth_name)
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("failed to set up container's network link: {}", e);

                        std::process::exit(1);
                    });
                net_if
                    .add_default_route("10.0.0.1".parse().unwrap())
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("failed to add default route for container: {}", e);

                        std::process::exit(1);
                    });
            });

            let _ = unistd::execvp(program, args);

            eprintln!("execvp failed");
            std::process::exit(127);
        }
        Err(e) => eprintln!("\nError: {}", e),
    }
}
