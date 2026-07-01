use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Reload,
    Pick,
    Scrape {
        tag: Option<String>,
        #[arg(short, long)]
        pick: bool,
    },
}
