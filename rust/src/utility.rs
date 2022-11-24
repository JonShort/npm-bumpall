pub enum UpgradeStyle {
    Latest,
    Wanted,
}

pub struct Config {
    pub legacy_peer_deps: bool,
    pub upgrade_style: UpgradeStyle,
}

impl Config {
    /// Accepts a list of arguments, usually an [Args][std::env::Args] struct
    /// sourced from the [std::env::args] function.
    pub fn new_from_args<T>(mut args: T) -> Result<Config, &'static str>
    where
        T: Iterator<Item = String>,
    {
        // ignore first arg (program name)
        args.next();

        let upgrade_style = match args.find(|x| x == "--latest" || x == "-l") {
            Some(_) => UpgradeStyle::Latest,
            None => UpgradeStyle::Wanted,
        };

        let legacy_peer_deps = args.any(|x| x == "--legacy-peer-deps" || x == "-lpd");

        Ok(Config {
            legacy_peer_deps,
            upgrade_style,
        })
    }
}
