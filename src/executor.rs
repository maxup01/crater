use std::ffi::{CString, CStr};
use nix::{ 
    sys::wait,
    unistd,
}; 

pub fn execute(program: &CStr, args: &[CString]) {
    const PROGRAM_NOT_FOUND_CODE: i32 = 127;

    match unsafe { unistd::fork() } {
        Ok(unistd::ForkResult::Parent { child, .. }) => {
            if let Ok(wait::WaitStatus::Exited(_, exit_code)) = wait::waitpid(child, None)
                && exit_code == PROGRAM_NOT_FOUND_CODE 
            {
                eprintln!("program not found");
            }
        },
        Ok(unistd::ForkResult::Child) => {
            let _ = unistd::execvp(program, args); 
        },
        Err(e) => eprintln!("\nError: {}", e)
    } 
}
