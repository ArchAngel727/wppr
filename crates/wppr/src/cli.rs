use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Reload,
    Scrape {
        tag: Option<String>,
        #[arg(short, long, value_parser = clap::value_parser!(bool))]
        pick: bool,
        #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..=3))]
        backstep: Option<u32>,
    },
}
