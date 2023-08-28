use std::process;

mod color_codes;
mod emojis;
mod npm_cmd;
mod package;
mod utility;

use emojis::{CACTUS, CROSS, DIZZY, MAGNIFYING_GLASS, POINT_RIGHT, ROCKET, TROPHY};
use package::{Package, UpgradeType};
use utility::{print_message, Config, UpgradeStyle};

#[cfg(windows)]
pub const NPM: &str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM: &str = "npm";

fn main() {
    let config = Config::create_config();

    print_message("Checking for outdated packages...", &MAGNIFYING_GLASS);

    let output = npm_cmd::run(&config).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(70)
    });

    let packages: Vec<Package> = output
        .iter()
        .filter_map(|(s, v)| match Package::new(s.into(), v, &config) {
            Ok(pkg) => {
                if pkg.skip {
                    None
                } else {
                    Some(pkg)
                }
            }
            Err(_) => None,
        })
        .collect();

    eprint!("{:?}", packages);

    if packages.is_empty() {
        println!("{} No outdated packages found {}", &ROCKET, &ROCKET);
        process::exit(0)
    }

    println!("Updates required");
    for pkg in packages.iter() {
        let upgrade_version = match &config.upgrade_style {
            UpgradeStyle::Latest => &pkg.latest_version,
            UpgradeStyle::Wanted => &pkg.wanted_version,
        };

        let color = match pkg.upgrade_type {
            UpgradeType::Safe => color_codes::CYAN,
            UpgradeType::Major => color_codes::YELLOW,
        };

        println!(
            "{} {} {} -> \x1b[{}m{}\x1b[0m",
            &POINT_RIGHT, pkg.name, pkg.current_version, color, upgrade_version
        );
    }
    println!();

    if config.is_dry_run {
        print_message("Dry run, exiting...", &CACTUS);
        process::exit(0);
    }

    let cmd_args: Vec<String> = packages
        .iter()
        .map(|pkg| String::from(&pkg.install_cmd))
        .collect();

    print_message("Upgrading packages", &DIZZY);

    let mut install = process::Command::new(NPM)
        .stdout(config.stdout_method)
        .stderr(config.stderr_method)
        .arg("i")
        .args(&cmd_args)
        .args(&config.additional_install_args)
        .spawn()
        .unwrap_or_else(|err| {
            eprintln!("{}", err);
            process::exit(70)
        });

    let status = install.wait().unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(70)
    });

    if status.success() {
        print_message("All packages bumped", &TROPHY);
    } else {
        print_message("Issue installing packages - try running manually", &CROSS);
    }
}
