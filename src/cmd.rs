use clap::Parser;
use std::any::type_name;
use std::process::Stdio;

/// Utility to bump npm packages, by default to the latest minor version.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   ///Bump dependencies to latest possible version (includes major changes)
   #[arg(short, long)]
   latest: bool,

   #[arg(short, long)]
   ///Update to latest patch version only
   patch: bool,

   #[arg(short = 'o', long)]
   ///Apply --legacy-peer-deps to npm install
   legacy_peer_deps: bool,

   #[arg(short, long)]
   ///Include all possible messages in console output (e.g. warnings from npm itself)
   verbose: bool,

   #[arg(short, long)]
   ///List dependencies which would be bumped, but don't update them
   dry_run: bool,
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
    pub fn new_from_args() -> Config {
        let args = Args::parse();

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