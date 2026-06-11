use std::{fs, io};

pub fn initialize_cgroup() -> Result<(), io::Error> {
    fs::create_dir_all("/sys/fs/cgroup/crater")?;

    fs::write(
        "/sys/fs/cgroup/cgroup.subtree_control",
        "+cpu +memory +pids",
    )?;
    fs::write("/sys/fs/cgroup/crater/cpu.max", "20000 100000")?;
    fs::write("/sys/fs/cgroup/crater/memory.max", "104857600")
}

pub fn add_process_to_cgroup(pid: i32) -> Result<(), io::Error> {
    fs::write("/sys/fs/cgroup/crater/cgroup.procs", pid.to_string())
}

pub fn cleanup_cgroup() -> Result<(), io::Error> {
    fs::remove_dir("/sys/fs/cgroup/crater")
}
