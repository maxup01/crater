use clap::Parser;
use std::ffi::{CStr, CString};

#[derive(Parser)]
#[command(name = "crater")]
pub struct Cli {
    #[arg(short, required = true)]
    image: String,

    #[arg(required = true, num_args = 1..)]
    args: Vec<CString>,
}

impl Cli {
    pub fn image(&self) -> &str {
        &self.image
    }

    pub fn program(&self) -> &CStr {
        &self.args[0]
    }

    pub fn args(&self) -> &[CString] {
        &self.args
    }
}
