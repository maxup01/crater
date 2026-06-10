use clap::Parser;
use crater::{execute, Cli};

fn main() {
    let args = Cli::parse();

    execute(args.program(), args.args());
}
