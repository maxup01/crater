use crate::store::StoreContext;
use error::CraterError;
use std::fs;

pub fn init() -> Result<(), CraterError> {
    StoreContext::init()?;

    initialize_cgroup()?;

    Ok(())
}

pub fn initialize_cgroup() -> Result<(), CraterError> {
    fs::create_dir_all("/sys/fs/cgroup/crater")?;

    fs::write(
        "/sys/fs/cgroup/cgroup.subtree_control",
        "+cpu +memory +pids",
    )?;
    fs::write("/sys/fs/cgroup/crater/cpu.max", "20000 100000")?;
    fs::write("/sys/fs/cgroup/crater/memory.max", "104857600")?;

    Ok(())
}

pub fn add_process_to_cgroup(pid: i32) -> Result<(), CraterError> {
    fs::write("/sys/fs/cgroup/crater/cgroup.procs", pid.to_string())?;

    Ok(())
}
