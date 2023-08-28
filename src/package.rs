use std::{error::Error, fmt};

use crate::{
    npm_cmd::PackageValue,
    utility::{Config, UpgradeStyle},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to parse input")
    }
}

impl Error for ParseError {}

#[derive(Debug, PartialEq, Eq)]
pub enum UpgradeType {
    Safe,
    Major,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Package {
    pub current_version: String,
    pub install_cmd: String,
    pub install_dir_name: String,
    pub latest_version: String,
    pub name: String,
    pub skip: bool,
    pub upgrade_type: UpgradeType,
    pub wanted_version: String,
}

impl Package {
    pub fn new(
        package_name: String,
        package_value: &PackageValue,
        config: &Config,
    ) -> Result<Package, ParseError> {
        if package_name.is_empty() {
            return Err(ParseError);
        };

        // On windows, the location starts with drive information, e.g. D:\\windows\dir
        // This clashes with the split on ":", so we handle this separately

        let name = package_name;
        let wanted_version = package_value.wanted.clone();
        let current_version = package_value.current.clone();
        let latest_version = package_value.latest.clone();
        let install_dir_name = package_value.dependent.clone();

        let upgrade_string = match config.upgrade_style {
            UpgradeStyle::Latest => latest_version.clone(),
            UpgradeStyle::Wanted => wanted_version.clone(),
        };
        eprint!("{:?}", install_dir_name);

        let install_cmd = format!("{}@{}", name, upgrade_string);
        let is_probably_workspace_dep = Some(install_dir_name.clone()) != config.current_dir_name;
        let skip = current_version == upgrade_string || is_probably_workspace_dep;
        let upgrade_type = match config.upgrade_style {
            UpgradeStyle::Wanted => UpgradeType::Safe,
            UpgradeStyle::Latest => {
                if wanted_version == latest_version {
                    UpgradeType::Safe
                } else {
                    UpgradeType::Major
                }
            }
        };

        Ok(Package {
            current_version,
            install_cmd,
            install_dir_name,
            latest_version,
            name,
            skip,
            upgrade_type,
            wanted_version,
        })
    }
}

// Tests --------------------------------------------------------------

#[cfg(test)]
mod package_tests {
    use crate::utility::Args;

    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    fn err_result_on_empty_string() {
        let config = Config::new_from_args(Args::default());
        let pkg = Package::new(String::from(""), &PackageValue::default(), &config);

        assert_eq!(pkg, Err(ParseError))
    }

    #[test]
    fn expected_result_on_valid_input_1() -> Result<(), ParseError> {
        let config = Config::new_from_args(Args::default());

        // location:name@wanted_version:name@current_version:name@latest_version
        let package_name = String::from("myPackage");
        let package_value = PackageValue {
            current: String::from("1.7.3"),
            wanted: String::from("1.23.0"),
            latest: String::from("2.0.1"),
            dependent: String::from("my_dir"),
            location: String::from("location"),
        };

        let pkg = Package::new(package_name, &package_value, &config)?;

        let expected = Package {
            current_version: String::from("1.7.3"),
            install_cmd: String::from("myPackage@1.23.0"),
            install_dir_name: String::from("my_dir"),
            latest_version: String::from("2.0.1"),
            name: String::from("myPackage"),
            skip: true,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("1.23.0"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_2() -> Result<(), ParseError> {
        let config = Config::new_from_args(Args {
            latest: true,
            ..Args::default()
        });

        // location:name@wanted_version:name@current_version:name@latest_version
        let package_name = String::from("myPackage");
        let package_value = PackageValue {
            current: String::from("1.7.3"),
            wanted: String::from("1.23.0"),
            latest: String::from("2.0.1"),
            dependent: String::from("dirNameThing"),
            location: String::from("location"),
        };

        let pkg = Package::new(package_name, &package_value, &config)?;

        let expected = Package {
            current_version: String::from("1.7.3"),
            install_cmd: String::from("myPackage@2.0.1"),
            install_dir_name: String::from("dirNameThing"),
            latest_version: String::from("2.0.1"),
            name: String::from("myPackage"),
            skip: true,
            upgrade_type: UpgradeType::Major,
            wanted_version: String::from("1.23.0"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_3() -> Result<(), ParseError> {
        let config = Config::new_from_args(Args::default());

        // location:name@wanted_version:name@current_version:name@latest_version
        let package_name = String::from("@jonshort/cenv");
        let package_value = PackageValue {
            current: String::from("125.24222.1"),
            wanted: String::from("125.24567.2"),
            latest: String::from("5412.0.0"),
            dependent: String::from("my-dir_with:special chars"),
            location: String::from("location"),
        };

        let pkg = Package::new(package_name, &package_value, &config)?;

        let expected = Package {
            current_version: String::from("125.24222.1"),
            install_cmd: String::from("@jonshort/cenv@125.24567.2"),
            install_dir_name: String::from("my-dir_with:special chars"),
            latest_version: String::from("5412.0.0"),
            name: String::from("@jonshort/cenv"),
            skip: true,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("125.24567.2"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_4() -> Result<(), ParseError> {
        let config = Config::new_from_args(Args {
            latest: true,
            ..Args::default()
        });

        // location:name@wanted_version:name@current_version:name@latest_version
        let package_name = String::from("@jonshort/cenv");
        let package_value = PackageValue {
            current: String::from("125.24222.1"),
            wanted: String::from("125.24567.2"),
            latest: String::from("5412.0.0"),
            dependent: String::from("a"),
            location: String::from("location"),
        };

        let pkg = Package::new(package_name, &package_value, &config)?;

        let expected = Package {
            current_version: String::from("125.24222.1"),
            install_cmd: String::from("@jonshort/cenv@5412.0.0"),
            install_dir_name: String::from("a"),
            latest_version: String::from("5412.0.0"),
            name: String::from("@jonshort/cenv"),
            skip: true,
            upgrade_type: UpgradeType::Major,
            wanted_version: String::from("125.24567.2"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    #[serial]
    fn expected_result_on_valid_input_5() -> Result<(), ParseError> {
        // worth setting the dir here as we need to ensure skip is true because of dep range
        let current = env::current_dir().unwrap();
        env::set_current_dir("./src/test_files").unwrap();

        let config = Config::new_from_args(Args { ..Args::default() });

        // location:name@wanted_version:name@current_version:name@latest_version
        let package_name = String::from("@jonshort/cenv");
        let package_value = PackageValue {
            current: String::from("1.0.2"),
            wanted: String::from("1.0.2"),
            latest: String::from("2.1.0"),
            dependent: String::from("test_files"),
            location: String::from("location"),
        };

        let pkg = Package::new(package_name, &package_value, &config)?;

        let expected = Package {
            current_version: String::from("1.0.2"),
            install_cmd: String::from("@jonshort/cenv@1.0.2"),
            install_dir_name: String::from("test_files"),
            latest_version: String::from("2.1.0"),
            name: String::from("@jonshort/cenv"),
            skip: true,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("1.0.2"),
        };

        assert_eq!(pkg, expected);

        env::set_current_dir(current).unwrap();
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_6() -> Result<(), ParseError> {
        let config = Config::new_from_args(Args {
            latest: true,
            ..Args::default()
        });

        // location:name@wanted_version:name@current_version:name@latest_version
        let package_name = String::from("@jonshort/cenv");
        let package_value = PackageValue {
            current: String::from("1.0.2"),
            wanted: String::from("1.0.3"),
            latest: String::from("1.0.3"),
            dependent: String::from("[]{}()dir*"),
            location: String::from("location"),
        };

        let pkg = Package::new(package_name, &package_value, &config)?;

        let expected = Package {
            current_version: String::from("1.0.2"),
            install_cmd: String::from("@jonshort/cenv@1.0.3"),
            install_dir_name: String::from("[]{}()dir*"),
            latest_version: String::from("1.0.3"),
            name: String::from("@jonshort/cenv"),
            skip: true,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("1.0.3"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_7() -> Result<(), ParseError> {
        let config = Config::new_from_args(Args {
            latest: true,
            ..Args::default()
        });
        // location:name@wanted_version:MISSING:name@latest_version
        let package_name = String::from("@jonshort/cenv");
        let package_value = PackageValue {
            current: String::from("MISSING"),
            wanted: String::from("1.0.3"),
            latest: String::from("1.0.3"),
            dependent: String::from("\\|~#;<>"),
            location: String::from("location"),
        };

        let pkg = Package::new(package_name, &package_value, &config)?;

        let expected = Package {
            current_version: String::from("MISSING"),
            install_cmd: String::from("@jonshort/cenv@1.0.3"),
            install_dir_name: String::from("\\|~#;<>"),
            latest_version: String::from("1.0.3"),
            name: String::from("@jonshort/cenv"),
            skip: true,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("1.0.3"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_windows() -> Result<(), ParseError> {
        let config = Config::new_from_args(Args {
            latest: true,
            ..Args::default()
        });

        // location:name@wanted_version:name@current_version:name@latest_version:project
        let package_name = String::from("@jonshort/cenv");
        let package_value = PackageValue {
            current: String::from("1.0.2"),
            wanted: String::from("1.0.3"),
            latest: String::from("1.0.3"),
            dependent: String::from("a"),
            location: String::from("D:\\git\npm:@jonshort/cenv@1.0.3"),
        };

        let pkg = Package::new(package_name, &package_value, &config)?;

        let expected = Package {
            current_version: String::from("1.0.2"),
            install_cmd: String::from("@jonshort/cenv@1.0.3"),
            install_dir_name: String::from("a"),
            latest_version: String::from("1.0.3"),
            name: String::from("@jonshort/cenv"),
            skip: true,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("1.0.3"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    #[serial]
    fn does_not_skip_direct_dep() -> Result<(), ParseError> {
        let current = env::current_dir().unwrap();
        env::set_current_dir("./src/test_files").unwrap();

        let config = Config::new_from_args(Args {
            latest: true,
            ..Args::default()
        });

        // location:name@wanted_version:name@current_version:name@latest_version:project
        // Also included \r which can be included on windows for some reason
        let package_name = String::from("@jonshort/cenv");
        let package_value = PackageValue {
            current: String::from("1.0.2"),
            wanted: String::from("1.0.3"),
            latest: String::from("1.0.3"),
            dependent: String::from("test_files"),
            location: String::from("D:\\git\npm:@jonshort/cenv@1.0.3"),
        };

        let pkg = Package::new(package_name, &package_value, &config)?;

        let expected = Package {
            current_version: String::from("1.0.2"),
            install_cmd: String::from("@jonshort/cenv@1.0.3"),
            install_dir_name: String::from("test_files"),
            latest_version: String::from("1.0.3"),
            name: String::from("@jonshort/cenv"),
            skip: false,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("1.0.3"),
        };
        assert_eq!(pkg, expected);

        env::set_current_dir(current).unwrap();
        Ok(())
    }
}
