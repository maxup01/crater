use clap::Parser;
use crater::Cli;

fn main() {
    crater::initialize_cgroup().expect("failed to create cgroup");

    let args = Cli::parse();

    crater::detach_process(move || {
        let args = args;

        crater::execute(args.image_path(), args.program(), args.args())
    });

    let _ = crater::cleanup_cgroup();
}
