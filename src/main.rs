use clap::Parser;
use crater::{execute, Cli};

fn main() {
    let args = Cli::parse();

    execute(args.image_path(), args.program(), args.args());
}
