use crate::Config;

pub struct App<'a> {
    pub config: Config,
    pub args: &'a [String],
}

impl<'a> App<'a> {
    pub fn new(config: Config, args: &'a [String]) -> App<'a> {
        App { config, args }
    }
}
