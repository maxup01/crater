use clap::Parser;
use std::ffi::{CStr, CString};

#[derive(Parser)]
#[command(name = "crater")]
pub struct Cli {
    #[arg(required = true)]
    program: CString,

    args: Vec<CString>,
}

impl Cli {
    pub fn program(&self) -> &CStr {
        &self.program
    }

    pub fn args(&self) -> &[CString] {
        &self.args
    }
}
