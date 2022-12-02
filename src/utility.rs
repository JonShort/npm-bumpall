use std::any::type_name;
use std::process::Stdio;

#[derive(PartialEq, Eq, Debug)]
pub enum UpgradeStyle {
    Latest,
    Wanted,
}

#[derive(Debug)]
pub struct Config {
    pub additional_install_args: Vec<String>,
    pub is_dry_run: bool,
    pub stderr_method: Stdio,
    pub stdout_method: Stdio,
    pub upgrade_style: UpgradeStyle,
}

fn print_type_of<T>(_: &T) -> &str {
    type_name::<T>()
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        let a = self.additional_install_args == other.additional_install_args;
        let dr = self.is_dry_run == other.is_dry_run;
        // This doesn't effectively check anything, but better than nothing
        let e = print_type_of(&self.stderr_method) == print_type_of(&other.stderr_method);
        let o = print_type_of(&self.stdout_method) == print_type_of(&other.stdout_method);
        let u = self.upgrade_style == other.upgrade_style;

        a && dr && e && o && u
    }
}

impl Config {
    /// Accepts a list of arguments, usually an [Args][std::env::Args] struct
    /// sourced from the [std::env::args] function.
    pub fn new_from_args<T>(args: T) -> Config
    where
        T: Iterator<Item = String>,
    {
        let mut additional_install_args = vec![];
        let mut is_dry_run = false;
        let mut stderr_method = Stdio::null();
        let mut stdout_method = Stdio::null();
        let mut upgrade_style = UpgradeStyle::Wanted;

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

            if arg == "--dry-run" || arg == "-dr" {
                is_dry_run = true;
            }
        }

        Config {
            additional_install_args,
            is_dry_run,
            stderr_method,
            stdout_method,
            upgrade_style,
        }
    }
}

pub fn print_message(message: &str, emoji: &char) {
    println!("{} {} {}", emoji, message, emoji);
    println!();
}

#[cfg(test)]
mod config_tests {
    use super::*;
    #[test]
    fn default_on_no_args() {
        let args = vec![];
        let result = Config::new_from_args(args.into_iter());
        let expected = Config {
            additional_install_args: vec![],
            is_dry_run: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result, expected)
    }

    #[test]
    fn handles_latest_arg() {
        let args_a = vec![String::from("--latest")];
        let result_a = Config::new_from_args(args_a.into_iter());
        let args_b = vec![String::from("-l")];
        let result_b = Config::new_from_args(args_b.into_iter());
        let expected = Config {
            additional_install_args: vec![],
            is_dry_run: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Latest,
        };
        assert_eq!(result_a, expected);
        assert_eq!(result_b, expected);
    }

    #[test]
    fn handles_legacy_deps_arg() {
        let args_a = vec![String::from("--legacy-peer-deps")];
        let result_a = Config::new_from_args(args_a.into_iter());
        let args_b = vec![String::from("-lpd")];
        let result_b = Config::new_from_args(args_b.into_iter());
        let expected = Config {
            additional_install_args: vec![String::from("--legacy-peer-deps")],
            is_dry_run: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
        assert_eq!(result_b, expected);
    }

    // doesn't actually check the value at the moment, but have left it here for completeness
    #[test]
    fn handles_verbose_arg() {
        let args_a = vec![String::from("--verbose")];
        let result_a = Config::new_from_args(args_a.into_iter());
        let args_b = vec![String::from("-vb")];
        let result_b = Config::new_from_args(args_b.into_iter());
        let expected = Config {
            additional_install_args: vec![],
            is_dry_run: false,
            stderr_method: Stdio::inherit(),
            stdout_method: Stdio::inherit(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
        assert_eq!(result_b, expected);
    }

    #[test]
    fn handles_dry_run_arg() {
        let args_a = vec![String::from("--dry-run")];
        let result_a = Config::new_from_args(args_a.into_iter());
        let args_b = vec![String::from("-dr")];
        let result_b = Config::new_from_args(args_b.into_iter());
        let expected = Config {
            additional_install_args: vec![],
            is_dry_run: true,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
        assert_eq!(result_b, expected);
    }

    #[test]
    fn handles_combo_args() {
        let args_a = vec![
            String::from("--latest"),
            String::from("--verbose"),
            String::from("-dr"),
            String::from("-lpd"),
        ];
        let result_a = Config::new_from_args(args_a.into_iter());
        let args_b = vec![
            String::from("--dry-run"),
            String::from("--legacy-peer-deps"),
            String::from("-l"),
            String::from("-vb"),
        ];
        let result_b = Config::new_from_args(args_b.into_iter());
        let expected = Config {
            additional_install_args: vec![String::from("--legacy-peer-deps")],
            is_dry_run: true,
            stderr_method: Stdio::inherit(),
            stdout_method: Stdio::inherit(),
            upgrade_style: UpgradeStyle::Latest,
        };
        assert_eq!(result_a, expected);
        assert_eq!(result_b, expected);
    }
}
