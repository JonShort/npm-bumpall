use clap::Parser;
use glob::Pattern;
use std::any::type_name;
use std::env::current_dir;
use std::process::Stdio;

/// Utility to bump npm packages, by default to the latest minor version.
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    ///Bump dependencies to latest possible version (includes major changes)
    #[arg(short, long)]
    pub latest: bool,

    #[arg(short, long)]
    ///Update to latest patch version only (experimental)
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

    #[arg(short, long)]
    ///Only bumps packages which match the glob pattern provided
    pub include: Option<String>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum UpgradeStyle {
    Latest,
    Wanted,
}

#[derive(Debug)]
pub struct Config {
    pub additional_install_args: Vec<String>,
    pub current_dir_name: Option<String>,
    pub include_glob: Option<Pattern>,
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
        let cdr = self.current_dir_name == other.current_dir_name;
        let dr = self.is_dry_run == other.is_dry_run;
        let pm = self.is_patch_mode == other.is_patch_mode;
        // This doesn't effectively check anything, but better than nothing
        let e = print_type_of(&self.stderr_method) == print_type_of(&other.stderr_method);
        let o = print_type_of(&self.stdout_method) == print_type_of(&other.stdout_method);
        let u = self.upgrade_style == other.upgrade_style;
        let i = self.include_glob == other.include_glob;

        a && cdr && dr && pm && e && o && u && i
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
        let mut include_glob = None;

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

        if let Some(g) = args.include {
            if let Ok(ptn) = Pattern::new(&g) {
                include_glob = Some(ptn);
            }
        }

        let current_dir_name = match current_dir().unwrap_or_default().file_name() {
            Some(d) => d.to_str().map(String::from),
            None => None,
        };

        Config {
            additional_install_args,
            current_dir_name,
            include_glob,
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

// Tests --------------------------------------------------------------

#[cfg(test)]
mod config_tests {
    use super::*;
    use glob::Pattern;
    use serial_test::{parallel, serial};
    use std::env;

    #[test]
    #[parallel]
    fn verify_cli() {
        use clap::CommandFactory;
        Args::command().debug_assert()
    }

    #[test]
    #[parallel]
    fn default_on_no_args() {
        let args = Args::default();
        let result = Config::new_from_args(args);
        let expected = Config {
            additional_install_args: vec![],
            current_dir_name: Some(String::from("npm-bumpall")),
            include_glob: None,
            is_dry_run: false,
            is_patch_mode: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result, expected)
    }

    #[test]
    #[parallel]
    fn handles_latest_arg() {
        let args_a = Args {
            latest: true,
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![],
            current_dir_name: Some(String::from("npm-bumpall")),
            include_glob: None,
            is_dry_run: false,
            is_patch_mode: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Latest,
        };
        assert_eq!(result_a, expected);
    }

    #[test]
    #[parallel]
    fn handles_legacy_deps_arg() {
        let args_a = Args {
            legacy_peer_deps: true,
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![String::from("--legacy-peer-deps")],
            current_dir_name: Some(String::from("npm-bumpall")),
            include_glob: None,
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
    #[parallel]
    fn handles_verbose_arg() {
        let args_a = Args {
            verbose: true,
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![],
            current_dir_name: Some(String::from("npm-bumpall")),
            include_glob: None,
            is_dry_run: false,
            is_patch_mode: false,
            stderr_method: Stdio::inherit(),
            stdout_method: Stdio::inherit(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
    }

    #[test]
    #[parallel]
    fn handles_dry_run_arg() {
        let args_a = Args {
            dry_run: true,
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![],
            current_dir_name: Some(String::from("npm-bumpall")),
            include_glob: None,
            is_dry_run: true,
            is_patch_mode: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
    }

    #[test]
    #[serial]
    fn handles_running_dir() {
        let current = env::current_dir().unwrap();
        env::set_current_dir("./src/test_files").unwrap();

        let result = Config::new_from_args(Args { ..Args::default() });
        let expected = Config {
            additional_install_args: vec![],
            current_dir_name: Some(String::from("test_files")),
            include_glob: None,
            is_dry_run: false,
            is_patch_mode: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };

        assert_eq!(result, expected);
        env::set_current_dir(current).unwrap();
    }

    #[test]
    #[parallel]
    fn handles_patch_mode_arg() {
        let args_a = Args {
            patch: true,
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![],
            current_dir_name: Some(String::from("npm-bumpall")),
            include_glob: None,
            is_dry_run: false,
            is_patch_mode: true,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
    }

    #[test]
    #[parallel]
    fn handles_include_arg() {
        let args_a = Args {
            include: Some(String::from("hello")),
            ..Args::default()
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![],
            current_dir_name: Some(String::from("npm-bumpall")),
            include_glob: Some(Pattern::new("hello").unwrap()),
            is_dry_run: false,
            is_patch_mode: false,
            stderr_method: Stdio::null(),
            stdout_method: Stdio::null(),
            upgrade_style: UpgradeStyle::Wanted,
        };
        assert_eq!(result_a, expected);
    }

    #[test]
    #[parallel]
    fn handles_combo_args() {
        let args_a = Args {
            dry_run: true,
            include: Some(String::from(".*")),
            latest: true,
            legacy_peer_deps: true,
            patch: true,
            verbose: true,
        };
        let result_a = Config::new_from_args(args_a);
        let expected = Config {
            additional_install_args: vec![String::from("--legacy-peer-deps")],
            current_dir_name: Some(String::from("npm-bumpall")),
            include_glob: Some(Pattern::new(".*").unwrap()),
            is_dry_run: true,
            is_patch_mode: true,
            stderr_method: Stdio::inherit(),
            stdout_method: Stdio::inherit(),
            upgrade_style: UpgradeStyle::Latest,
        };
        assert_eq!(result_a, expected);
    }
}
