use std::{env, process};

mod emojis;
mod package;
mod utility;

use emojis::{CROSS, DIZZY, MAGNIFYING_GLASS, POINT_RIGHT, ROCKET, TROPHY};
use package::Package;
use utility::{Config, UpgradeStyle};

fn main() {
    let config = Config::new_from_args(env::args()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(65);
    });

    println!(
        "{} Checking for outdated packages... {}",
        &MAGNIFYING_GLASS, &MAGNIFYING_GLASS
    );
    println!();

    let output = process::Command::new("npm")
        .arg("outdated")
        .arg("--parseable")
        .output()
        .expect("Failed running npm script!");

    let output = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(65)
        }
    };

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

    let mut cmd_args: Vec<String> = packages
        .iter()
        .map(|pkg| String::from(&pkg.install_cmd))
        .collect();

    if config.legacy_peer_deps {
        cmd_args.push(String::from("--legacy-peer-deps"));
    }

    println!("{} Upgrading packages {}", &DIZZY, &DIZZY);
    println!();

    let mut install = process::Command::new("npm")
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .arg("i")
        .args(&cmd_args)
        .spawn()
        .expect("Failed running npm script!");

    let status = install.wait().expect("npm script failed");

    if status.success() {
        println!("{} All packages bumped {}", &TROPHY, &TROPHY);
    } else {
        println!(
            "{} Issue installing packages - try running manually {}",
            &CROSS, &CROSS
        );
    }
}
