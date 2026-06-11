use clap::Parser;
use crater::{Cli, Command};

fn main() {
    crater::init().expect("failed initialize crater");

    let cli = Cli::parse();

    match cli.command {
        Command::Run { image_path, args } => {
            crater::detach_process(move || {
                let image_path = image_path;
                let args = args;

                crater::execute(&image_path, &args[0], &args);
            });
        }
        Command::List { list_images } => {
            if list_images {
                crater::list_images();
            }
        }
    }
}
