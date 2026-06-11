use clap::{Parser, Subcommand};
use std::{ffi::CString, path::PathBuf};

#[derive(Parser)]
#[command(name = "crater")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Run {
        #[arg(short, required = true)]
        image_path: PathBuf,

        #[arg(required = true, num_args = 1..)]
        args: Vec<CString>,
    },
    List {
        #[arg(short = 'i')]
        list_images: bool,
    },
}
