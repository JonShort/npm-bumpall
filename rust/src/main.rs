use std::process;

mod emojis;
mod package;
mod utility;

use emojis::{CROSS, DIZZY, MAGNIFYING_GLASS, POINT_RIGHT, ROCKET, TROPHY};
use package::Package;

fn main() {
    let dir = utility::get_var("DIR", ".");

    println!(
        "{} Checking for outdated packages... {}",
        &MAGNIFYING_GLASS, &MAGNIFYING_GLASS
    );
    println!();

    let output = process::Command::new("npm")
        .arg("outdated")
        .arg("--parseable")
        .current_dir(&dir)
        .output()
        .expect("Failed running npm script!");

    let output = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            process::exit(65)
        }
    };

    if output.trim() == "" {
        println!("{} No outdated packages found {}", &ROCKET, &ROCKET);
        process::exit(0)
    }

    let split_by_eol: Vec<&str> = output.split_terminator('\n').collect();

    let packages: Vec<Package> = split_by_eol
        .iter()
        .filter_map(|&s| {
            let pkg = Package::new(s.into());

            match pkg {
                Ok(p) => Some(p),
                Err(_) => None,
            }
        })
        .collect();

    println!("Updates required");
    for pkg in packages.iter() {
        println!(
            "{} {} {} -> {}",
            &POINT_RIGHT, pkg.name, pkg.current_version, pkg.latest_version
        );
    }
    println!();

    let cmd_args: Vec<String> = packages
        .iter()
        .map(|pkg| String::from(&pkg.install_cmd))
        .collect();

    println!("{} Upgrading packages {}", &DIZZY, &DIZZY);
    println!();

    let mut install = process::Command::new("npm")
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .arg("i")
        .args(&cmd_args)
        .current_dir(&dir)
        .spawn()
        .expect("Failed running npm script!");

    let status = install.wait().expect("npm script failed");

    if status.success() {
        println!("{} All packages now bumped to latest {}", &TROPHY, &TROPHY);
    } else {
        println!(
            "{} Issue installing packages - try running manually {}",
            &CROSS, &CROSS
        );
    }
}
