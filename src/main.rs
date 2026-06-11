use clap::Parser;
use crater::{Cli, Command};

fn main() {
    crater::init().expect("failed to initialize crater");

    let cli = Cli::parse();

    match cli.command {
        Command::Run {
            image,
            detach,
            args,
        } => {
            if detach {
                crater::detach_process(move || {
                    let image = image;
                    let args = args;

                    crater::execute(&image, &args[0], &args);
                });
            } else {
                crater::execute(&image, &args[0], &args);
            }
        }
        Command::List { list_images } => {
            if list_images {
                crater::list_images();
            }
        }
    }
}
