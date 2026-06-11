use std::{fs, io};

pub const IMAGE_STORE_DIRECTORY_PATH: &str = "/crater/images";

pub fn init() -> Result<(), io::Error> {
    fs::create_dir_all(IMAGE_STORE_DIRECTORY_PATH)?;

    initialize_cgroup()
}

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
