use clap::Parser;
use crater::{Cli, Command, StoreContext};

fn main() {
    crater::init().expect("failed to initialize crater");

    let cli = Cli::parse();

    match cli.command {
        Command::Create { image, name, args } => {
            crater::create_container(name, image, args);
        }
        Command::Run { name, detach } => {
            crater::run_container(name, detach);
        }
        Command::List { list_images } => {
            if list_images {
                StoreContext::list_images();
            }
        }
    }
}
