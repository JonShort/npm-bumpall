use std::{env, process};

mod emojis;
mod package;
mod utility;

use emojis::{CROSS, DIZZY, MAGNIFYING_GLASS, POINT_RIGHT, ROCKET, TROPHY};
use package::Package;
use utility::{print_message, Config, UpgradeStyle};

fn main() {
    let config = Config::new_from_args(env::args());

    print_message("Checking for outdated packages...", &MAGNIFYING_GLASS);

    let output = process::Command::new("npm")
        .arg("outdated")
        .arg("--parseable")
        .output()
        .unwrap_or_else(|err| {
            eprintln!("{}", err);
            process::exit(70)
        });

    let output = String::from_utf8(output.stdout).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(70)
    });

    let split_by_eol: Vec<&str> = output.split_terminator('\n').collect();
    let packages: Vec<Package> = split_by_eol
        .iter()
        .filter_map(|&s| match Package::new(s.into(), &config) {
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

        println!(
            "{} {} {} -> {}",
            &POINT_RIGHT, pkg.name, pkg.current_version, upgrade_version
        );
    }
    println!();

    let cmd_args: Vec<String> = packages
        .iter()
        .map(|pkg| String::from(&pkg.install_cmd))
        .collect();

    print_message("Upgrading packages", &DIZZY);

    let mut install = process::Command::new("npm")
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
