use clap::Parser;
use crater::{Cleaner, Cli, Command, StoreContext};

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
        Command::List {
            list_images,
            list_containers,
        } => {
            if list_images {
                StoreContext::list_images();
            } else if list_containers {
                StoreContext::list_containers();
            }
        }
        Command::Delete { image, container } => {
            if let Some(image) = image {
                Cleaner::remove_image(image.as_str());
            } else if let Some(container) = container {
                Cleaner::remove_container(container.as_str());
            }
        }
    }
}
