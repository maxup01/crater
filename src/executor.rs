use nix::{
    mount::{self, MntFlags, MsFlags},
    sched::{self, CloneFlags},
    sys::wait::{self, WaitStatus},
    unistd::{self, ForkResult},
};
use std::{
    ffi::{CStr, CString},
    fs::create_dir_all,
};

pub fn execute(image: &str, program: &CStr, args: &[CString]) {
    let flags = CloneFlags::CLONE_NEWUTS
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWNET
        | CloneFlags::CLONE_NEWUSER;

    if let Err(e) = sched::unshare(flags) {
        eprintln!("unshare failed: {e}");

        return;
    }

    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { child, .. }) => match wait::waitpid(child, None) {
            Ok(WaitStatus::Exited(_, code)) => {
                if code != 0 {
                    eprintln!("container exited with code {code}");
                }
            }
            Ok(other) => eprintln!("container ended: {other:?}"),
            Err(e) => eprintln!("waitpid failed: {e}"),
        },
        Ok(ForkResult::Child) => {
            let new_root = format!("/containers/{}", image);
            let old_root_temp = format!("{}/oldroot", &new_root);

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

            if let Err(e) = create_dir_all(new_root.as_str()) {
                eprintln!("failed to create new root directory: {}", e);
            }

            if let Err(e) = mount::mount(
                Some(new_root.as_str()),
                new_root.as_str(),
                None::<&str>,
                MsFlags::MS_BIND | MsFlags::MS_REC,
                None::<&str>,
            ) {
                eprintln!("failed to mount root directory: {}", e);
            }

            if let Err(e) = std::fs::create_dir_all(old_root_temp.as_str()) {
                eprintln!(
                    "failed to create temporary directory for previous root directory: {}",
                    e
                );
            }

            if let Err(e) = unistd::pivot_root(new_root.as_str(), old_root_temp.as_str()) {
                eprintln!("failed to change root directory: {}", e);
            }

            if let Err(e) = unistd::chdir("/") {
                eprintln!("failed to navigate to a new root directory: {}", e);
            }

            if let Err(e) = mount::umount2("/oldroot", MntFlags::MNT_DETACH) {
                eprintln!("failed to unmount oldroot directory: {}", e);
            }

            let _ = unistd::execvp(program, args);

            eprintln!("execvp failed");
            std::process::exit(127);
        }
        Err(e) => eprintln!("\nError: {}", e),
    }
}
