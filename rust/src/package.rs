use std::{error::Error, fmt};

use crate::utility::{Config, UpgradeStyle};

#[derive(Debug, Clone)]
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

fn split_name_and_version(src: Option<&str>) -> Result<(String, String), ParseError> {
    let src = val_or_err(src)?;

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

pub struct Package {
    pub current_version: String,
    pub install_cmd: String,
    pub latest_version: String,
    pub name: String,
    pub skip: bool,
    pub wanted_version: String,
}

impl Package {
    pub fn new(src: String, config: &Config) -> Result<Package, ParseError> {
        // location:name@current_version:name@wanted_version:name@latest_version:project
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

        Ok(Package {
            current_version,
            install_cmd,
            latest_version,
            name,
            skip,
            wanted_version,
        })
    }
}
