use std::path::Path;

use crate::cli::Cli;
use crate::Config;

pub struct App<'a> {
    pub config_path: &'a Path,
    pub config: Config,
    pub args: Cli,
}

impl<'a> App<'a> {
    pub fn new(config_path: &'a Path, config: Config, args: Cli) -> App<'a> {
        App {
            config_path,
            config,
            args,
        }
    }
}
