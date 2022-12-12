use std::{error::Error, fmt};

use crate::utility::{Config, UpgradeStyle};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to parse input")
    }
}

impl Error for ParseError {}

fn val_or_err<T>(opt: Option<T>) -> Result<T, ParseError> {
    if let Some(val) = opt {
        Ok(val)
    } else {
        Err(ParseError)
    }
}

const MISSING: &str = "MISSING";

fn split_name_and_version(src: Option<&str>) -> Result<(String, String), ParseError> {
    let src = val_or_err(src)?;

    if src == MISSING {
        return Ok((String::from(""), String::from(MISSING)));
    }

    let is_scoped_package = src.starts_with('@');
    let mut segments = src.split('@');

    if is_scoped_package {
        // the first value is guaranteed to be an empty slice
        segments.next();
    }

    let name = val_or_err(segments.next())?;
    let version = val_or_err(segments.next())?;

    if name.trim().is_empty() || version.trim().is_empty() {
        return Err(ParseError);
    }

    let prefix = if is_scoped_package { "@" } else { "" };

    Ok((format!("{}{}", prefix, name), version.to_string()))
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpgradeType {
    Safe,
    Major,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Package {
    pub current_version: String,
    pub install_cmd: String,
    pub latest_version: String,
    pub name: String,
    pub skip: bool,
    pub upgrade_type: UpgradeType,
    pub wanted_version: String,
}

impl Package {
    pub fn new(src: String, config: &Config) -> Result<Package, ParseError> {
        // :name@wanted_version:MISSING:name@latest_version:project
        // location:name@wanted_version:name@current_version:name@latest_version:project
        let mut segments = src.split(':');
        let _location = val_or_err(segments.next())?;
        let (name, wanted_version) = split_name_and_version(segments.next())?;
        let (_, current_version) = split_name_and_version(segments.next())?;
        let (_, latest_version) = split_name_and_version(segments.next())?;

        let upgrade_string = match config.upgrade_style {
            UpgradeStyle::Latest => latest_version.clone(),
            UpgradeStyle::Wanted => wanted_version.clone(),
        };

        let install_cmd = format!("{}@{}", name, upgrade_string);
        let skip = current_version == upgrade_string;
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
mod val_or_err_tests {
    use super::*;

    #[test]
    fn err_result_on_none() {
        assert_eq!(val_or_err::<Option<()>>(None), Err(ParseError));
    }

    #[test]
    fn ok_val_on_some() {
        assert_eq!(val_or_err(Some("hello")), Ok("hello"));
        assert_eq!(val_or_err(Some(false)), Ok(false));
        assert_eq!(val_or_err(Some(ParseError)), Ok(ParseError));
    }
}

#[cfg(test)]
mod split_name_and_version_tests {
    use super::*;

    #[test]
    fn err_result_on_invalid_input() {
        let test_cases = vec![
            None,
            Some(""),
            Some("noversion@"),
            Some("@0.5.5"),
            Some("@jonshort/cenv0.1.0"),
        ];

        for case in test_cases {
            assert_eq!(split_name_and_version(case), Err(ParseError));
        }
    }

    #[test]
    fn correct_result_on_scoped() {
        assert_eq!(
            split_name_and_version(Some("@jonshort/cenv@0.1.0")),
            Ok((String::from("@jonshort/cenv"), String::from("0.1.0")))
        );
    }

    #[test]
    fn correct_result_non_scoped() {
        assert_eq!(
            split_name_and_version(Some("package-name@0.1.0")),
            Ok((String::from("package-name"), String::from("0.1.0")))
        );
    }
}

#[cfg(test)]
mod package_tests {
    use super::*;

    #[test]
    fn err_result_on_empty_string() {
        let args = vec![];
        let config = Config::new_from_args(args.into_iter());
        let pkg = Package::new(String::from(""), &config);

        assert_eq!(pkg, Err(ParseError))
    }

    #[test]
    fn err_result_on_invalid_string() {
        // valid input string:
        // location:name@wanted_version:name@current_version:name@latest_version
        let test_cases = vec![
            String::from("location:name@2.0.0:name@1.0.0:name@"),
            String::from("location:name@2.0.0:name@1.0.0:name"),
            String::from("location:name@2.0.0:name@1.0.0:"),
            String::from("location:name@2.0.0:name@"),
            String::from("location:name@2.0.0:name"),
            String::from("location:name@2.0.0:"),
            String::from("location:name@"),
            String::from("location:name"),
            String::from("location:"),
            String::from("loc"),
            String::from(""),
        ];

        for case in test_cases {
            let args = vec![];
            let config = Config::new_from_args(args.into_iter());
            let pkg = Package::new(case, &config);

            assert_eq!(pkg, Err(ParseError))
        }
    }

    #[test]
    fn expected_result_on_valid_input_1() -> Result<(), ParseError> {
        let args = vec![];
        let config = Config::new_from_args(args.into_iter());
        // location:name@wanted_version:name@current_version:name@latest_version
        let provided = String::from("location:myPackage@1.23.0:myPackage@1.7.3:myPackage@2.0.1");
        let pkg = Package::new(provided, &config)?;

        let expected = Package {
            upgrade_type: UpgradeType::Safe,
            current_version: String::from("1.7.3"),
            install_cmd: String::from("myPackage@1.23.0"),
            latest_version: String::from("2.0.1"),
            name: String::from("myPackage"),
            skip: false,
            wanted_version: String::from("1.23.0"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_2() -> Result<(), ParseError> {
        let args = vec![String::from("--latest")];
        let config = Config::new_from_args(args.into_iter());
        // location:name@wanted_version:name@current_version:name@latest_version
        let provided = String::from("location:myPackage@1.23.0:myPackage@1.7.3:myPackage@2.0.1");
        let pkg = Package::new(provided, &config)?;

        let expected = Package {
            current_version: String::from("1.7.3"),
            install_cmd: String::from("myPackage@2.0.1"),
            latest_version: String::from("2.0.1"),
            name: String::from("myPackage"),
            skip: false,
            upgrade_type: UpgradeType::Major,
            wanted_version: String::from("1.23.0"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_3() -> Result<(), ParseError> {
        let args = vec![];
        let config = Config::new_from_args(args.into_iter());
        // location:name@wanted_version:name@current_version:name@latest_version
        let provided = String::from("location:@jonshort/cenv@125.24567.2:@jonshort/cenv@125.24222.1:@jonshort/cenv@5412.0.0");
        let pkg = Package::new(provided, &config)?;

        let expected = Package {
            current_version: String::from("125.24222.1"),
            install_cmd: String::from("@jonshort/cenv@125.24567.2"),
            latest_version: String::from("5412.0.0"),
            name: String::from("@jonshort/cenv"),
            skip: false,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("125.24567.2"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_4() -> Result<(), ParseError> {
        let args = vec![String::from("--latest")];
        let config = Config::new_from_args(args.into_iter());
        // location:name@wanted_version:name@current_version:name@latest_version
        let provided = String::from("location:@jonshort/cenv@125.24567.2:@jonshort/cenv@125.24222.1:@jonshort/cenv@5412.0.0");
        let pkg = Package::new(provided, &config)?;

        let expected = Package {
            current_version: String::from("125.24222.1"),
            install_cmd: String::from("@jonshort/cenv@5412.0.0"),
            latest_version: String::from("5412.0.0"),
            name: String::from("@jonshort/cenv"),
            skip: false,
            upgrade_type: UpgradeType::Major,
            wanted_version: String::from("125.24567.2"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_5() -> Result<(), ParseError> {
        let args = vec![];
        let config = Config::new_from_args(args.into_iter());
        // location:name@wanted_version:name@current_version:name@latest_version
        let provided =
            String::from("location:@jonshort/cenv@1.0.2:@jonshort/cenv@1.0.2:@jonshort/cenv@2.1.0");
        let pkg = Package::new(provided, &config)?;

        let expected = Package {
            current_version: String::from("1.0.2"),
            install_cmd: String::from("@jonshort/cenv@1.0.2"),
            latest_version: String::from("2.1.0"),
            name: String::from("@jonshort/cenv"),
            skip: true,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("1.0.2"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_6() -> Result<(), ParseError> {
        let args = vec![String::from("--latest")];
        let config = Config::new_from_args(args.into_iter());
        // location:name@wanted_version:name@current_version:name@latest_version
        let provided =
            String::from("location:@jonshort/cenv@1.0.3:@jonshort/cenv@1.0.2:@jonshort/cenv@1.0.3");
        let pkg = Package::new(provided, &config)?;

        let expected = Package {
            current_version: String::from("1.0.2"),
            install_cmd: String::from("@jonshort/cenv@1.0.3"),
            latest_version: String::from("1.0.3"),
            name: String::from("@jonshort/cenv"),
            skip: false,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("1.0.3"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }

    #[test]
    fn expected_result_on_valid_input_7() -> Result<(), ParseError> {
        let args = vec![String::from("--latest")];
        let config = Config::new_from_args(args.into_iter());
        // location:name@wanted_version:MISSING:name@latest_version
        let provided =
            String::from("location:@jonshort/cenv@1.0.3:MISSING:@jonshort/cenv@1.0.3");
        let pkg = Package::new(provided, &config)?;

        let expected = Package {
            current_version: String::from("MISSING"),
            install_cmd: String::from("@jonshort/cenv@1.0.3"),
            latest_version: String::from("1.0.3"),
            name: String::from("@jonshort/cenv"),
            skip: false,
            upgrade_type: UpgradeType::Safe,
            wanted_version: String::from("1.0.3"),
        };
        assert_eq!(pkg, expected);
        Ok(())
    }
}
