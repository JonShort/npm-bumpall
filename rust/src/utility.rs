use std::process::Stdio;

pub enum UpgradeStyle {
    Latest,
    Wanted,
}

pub struct Config {
    pub additional_install_args: Vec<String>,
    pub stderr_method: Stdio,
    pub stdout_method: Stdio,
    pub upgrade_style: UpgradeStyle,
}

impl Config {
    /// Accepts a list of arguments, usually an [Args][std::env::Args] struct
    /// sourced from the [std::env::args] function.
    pub fn new_from_args<T>(args: T) -> Result<Config, &'static str>
    where
        T: Iterator<Item = String>,
    {
        let mut stderr_method = Stdio::null();
        let mut additional_install_args = vec![];
        let mut upgrade_style = UpgradeStyle::Wanted;
        let mut stdout_method = Stdio::null();

        for arg in args {
            if arg == "--latest" || arg == "-l" {
                upgrade_style = UpgradeStyle::Latest;
                continue;
            }

            if arg == "--legacy-peer-deps" || arg == "-lpd" {
                additional_install_args.push(String::from("--legacy-peer-deps"));
                continue;
            }

            if arg == "--verbose" || arg == "-vb" {
                stdout_method = Stdio::inherit();
                stderr_method = Stdio::inherit();
                continue;
            }
        }

        Ok(Config {
            additional_install_args,
            stderr_method,
            stdout_method,
            upgrade_style,
        })
    }
}

pub fn print_message(message: &str, emoji: &char) {
    println!("{} {} {}", emoji, message, emoji);
    println!();
}
