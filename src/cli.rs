use clap::Parser;
use std::{
    ffi::{CStr, CString},
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(name = "crater")]
pub struct Cli {
    #[arg(short, required = true)]
    image_path: PathBuf,

    #[arg(required = true, num_args = 1..)]
    args: Vec<CString>,
}

impl Cli {
    pub fn image_path(&self) -> &Path {
        &self.image_path
    }

    pub fn program(&self) -> &CStr {
        &self.args[0]
    }

    pub fn args(&self) -> &[CString] {
        &self.args
    }
}
