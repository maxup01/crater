use clap::{ArgGroup, Parser, Subcommand};
use std::ffi::CString;

#[derive(Parser)]
#[command(name = "crater")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Create {
        #[arg(short, required = true)]
        image: String,

        #[arg(short, required = true)]
        name: String,

        #[arg(required = true, num_args = 1..)]
        args: Vec<CString>,
    },
    Run {
        #[arg(required = true)]
        name: String,

        #[arg(short)]
        detach: bool,
    },

    #[command(group(
        ArgGroup::new("list_kind")
            .required(true)
            .args(["list_images", "list_containers"])
    ))]
    List {
        #[arg(short = 'i')]
        list_images: bool,

        #[arg(short = 'c')]
        list_containers: bool,
    },
}
