use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to parse input")
    }
}

impl Error for ParseError {}

pub struct Package {
    pub current_version: String,
    pub install_cmd: String,
    pub latest_version: String,
    pub name: String,
    pub wanted_version: String,
}

fn val_or_err<T>(opt: Option<T>) -> Result<T, ParseError> {
    if let Some(val) = opt {
        Ok(val)
    } else {
        Err(ParseError)
    }
}

fn split_name_and_version(src: Option<&str>) -> Result<(String, String), ParseError> {
    let src = val_or_err(src)?;
    let mut segments = src.split('@');

    let name = val_or_err(segments.next())?;
    let version = val_or_err(segments.next())?;

    Ok((name.to_string(), version.to_string()))
}

impl Package {
    pub fn new(src: String) -> Result<Package, ParseError> {
        // location:name@current_version:name@wanted_version:name@latest_version:project
        let mut segments = src.split(':');
        let _location = val_or_err(segments.next())?;
        let (name, wanted_version) = split_name_and_version(segments.next())?;
        let (_, current_version) = split_name_and_version(segments.next())?;
        let (_, latest_version) = split_name_and_version(segments.next())?;
        let install_cmd = format!("{}@latest", name);

        Ok(Package {
            name,
            current_version,
            wanted_version,
            latest_version,
            install_cmd,
        })
    }
}
