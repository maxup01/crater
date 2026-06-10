use nix::{
    sched::{self, CloneFlags},
    sys::wait::{self, WaitStatus},
    unistd::{self, ForkResult},
};
use std::ffi::{CStr, CString};

pub fn execute(program: &CStr, args: &[CString]) {
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
            let _ = unistd::execvp(program, args);
        }
        Err(e) => eprintln!("\nError: {}", e),
    }
}
