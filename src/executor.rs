use crate::{
    network::{ContainerSideInterface, HostSideInterface, NetworkInterface},
    store::StoreContext,
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

pub fn detach_process<F: FnOnce()>(action: F) {
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

pub fn execute(image: &str, program: &CStr, args: &[CString]) {
    let flags = CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS;

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
            }
        }
        Ok(ForkResult::Child) => {
            if let Err(e) = sched::unshare(CloneFlags::CLONE_NEWNET) {
                eprintln!("failed to make child's network namespace: {}", e);
            }

            let _ = unistd::write(c_write, &[1u8]);

            let mut b = [0u8; 1];
            let _ = unistd::read(c_read, &mut b);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let image_path = PathBuf::from(StoreContext::image_store_dir()).join(image);
            let old_root_temp = image_path.join("oldroot");

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

            if let Err(e) = mount::mount(
                Some(&image_path),
                &image_path,
                None::<&str>,
                MsFlags::MS_BIND | MsFlags::MS_REC,
                None::<&str>,
            ) {
                eprintln!("failed to mount root directory: {}", e);
            }

            if let Err(e) = fs::create_dir_all(&old_root_temp) {
                eprintln!(
                    "failed to create temporary directory for previous root directory: {}",
                    e
                );
            }

            if let Err(e) = unistd::pivot_root(&image_path, &old_root_temp) {
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
