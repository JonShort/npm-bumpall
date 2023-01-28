use clap::Parser;
use std::any::type_name;
use std::process::Stdio;

/// Utility to bump npm packages, by default to the latest minor version.
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    ///Bump dependencies to latest possible version (includes major changes)
    #[arg(short, long)]
    pub latest: bool,

    #[arg(short, long)]
    ///Update to latest patch version only
    pub patch: bool,

    #[arg(long)]
    ///Apply --legacy-peer-deps to npm install
    pub legacy_peer_deps: bool,

    #[arg(short, long)]
    ///Include all possible messages in console output (e.g. warnings from npm itself)
    pub verbose: bool,

    #[arg(short, long)]
    ///List dependencies which would be bumped, but don't update them
    pub dry_run: bool,
}

#[derive(PartialEq, Eq, Debug)]
pub enum UpgradeStyle {
    Latest,
    Wanted,
}

#[derive(Debug)]
pub struct Config {
    pub additional_install_args: Vec<String>,
    pub is_dry_run: bool,
    pub is_patch_mode: bool,
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
        let pm = self.is_patch_mode == other.is_patch_mode;
        // This doesn't effectively check anything, but better than nothing
        let e = print_type_of(&self.stderr_method) == print_type_of(&other.stderr_method);
        let o = print_type_of(&self.stdout_method) == print_type_of(&other.stdout_method);
        let u = self.upgrade_style == other.upgrade_style;

        a && dr && pm && e && o && u
    }
}

impl Config {
    pub fn create_config() -> Config {
        let args = Args::parse();
        Self::new_from_args(args)
    }
    pub fn new_from_args(args: Args) -> Config {
        let mut additional_install_args = vec![];
        let mut stderr_method = Stdio::null();
        let mut stdout_method = Stdio::null();
        let mut upgrade_style = UpgradeStyle::Wanted;

        if args.latest {
            upgrade_style = UpgradeStyle::Latest;
        }

        if args.verbose {
            stdout_method = Stdio::inherit();
            stderr_method = Stdio::inherit();
        }

        if args.legacy_peer_deps {
            additional_install_args.push(String::from("--legacy-peer-deps"));
        }

        Config {
            additional_install_args,
            is_dry_run: args.dry_run,
            is_patch_mode: args.patch,
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
    fn verify_cli() {
        use clap::CommandFactory;
        Args::command().debug_assert()
    }

    #[test]
    fn default_on_no_args() {
        let args = Args::default();
        let result = Config::new_from_args(args);
        let expected = Config {
            additional_install_args: vec![],
            is_dry_run: false,
            is_patch_mode: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result, expected)
    }

    #[test]
    fn handles_latest_arg() {
        let args_a = Args {
            latest: true,
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![],
            is_dry_run: false,
            is_patch_mode: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Latest,
        };
        assert_eq!(result_a, expected);
    }

    #[test]
    fn handles_legacy_deps_arg() {
        let args_a = Args {
            legacy_peer_deps: true,
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![String::from("--legacy-peer-deps")],
            is_dry_run: false,
            is_patch_mode: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
    }

    // doesn't actually check the value at the moment, but have left it here for completeness
    #[test]
    fn handles_verbose_arg() {
        let args_a = Args {
            verbose: true,
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![],
            is_dry_run: false,
            is_patch_mode: false,
            stderr_method: Stdio::inherit(),
            stdout_method: Stdio::inherit(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
    }

    #[test]
    fn handles_dry_run_arg() {
        let args_a = Args {
            dry_run: true,
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![],
            is_dry_run: true,
            is_patch_mode: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
    }

    #[test]
    fn handles_patch_mode_arg() {
        let args_a = Args {
            patch: true,
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![],
            is_dry_run: false,
            is_patch_mode: true,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
    }

    #[test]
    fn handles_combo_args() {
        let args_a = Args {
            latest: true,
            patch: true,
            verbose: true,
            dry_run: true,
            legacy_peer_deps: true,
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![String::from("--legacy-peer-deps")],
            is_dry_run: true,
            is_patch_mode: true,
            stderr_method: Stdio::inherit(),
            stdout_method: Stdio::inherit(),
            upgrade_style: UpgradeStyle::Latest,
        };
        assert_eq!(result_a, expected);
    }
}
