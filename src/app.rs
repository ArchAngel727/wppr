use std::path::Path;

use crate::Config;

pub struct App<'a> {
    pub config_path: &'a Path,
    pub config: Config,
    pub args: &'a [String],
}

impl<'a> App<'a> {
    pub fn new(config_path: &'a Path, config: Config, args: &'a [String]) -> App<'a> {
        App {
            config_path,
            config,
            args,
        }
    }
}
