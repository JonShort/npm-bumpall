use serde_json::{Map, Value};
use std::{error::Error, fs, process};

use crate::utility::Config;

#[cfg(windows)]
pub const NPM: &str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM: &str = "npm";

fn prefix_with_tilde(pkg_version: &str) -> String {
    let mut chars = pkg_version.chars();
    let string_to_prefix: String = match chars.next().unwrap_or('a') {
        '^' => chars.collect(),
        _ => String::from(pkg_version),
    };

    format!("~{}", string_to_prefix)
}

fn prefix_all_entries_with_tilde(obj: &mut Value, dep_section: &str) {
    if let Some(deps) = obj.get_mut(dep_section) {
        let deps = match deps.as_object_mut() {
            Some(d) => d,
            None => {
                return;
            }
        };

        let mut new_deps = Map::new();

        for (key, val) in deps.iter() {
            let new_val = match val.as_str() {
                Some(v) => Value::from(prefix_with_tilde(v)),
                None => val.clone(),
            };

            new_deps.insert(key.into(), new_val);
        }

        obj[dep_section] = new_deps.into();
    }
}

fn patch_mode_init() -> Result<(), Box<dyn Error>> {
    fs::copy("package.json", "package.json.bkup")?;

    // write new package.json
    let pkg = fs::read_to_string("package.json")?;

    let mut v: Value = serde_json::from_str(&pkg)?;

    prefix_all_entries_with_tilde(&mut v, "dependencies");
    prefix_all_entries_with_tilde(&mut v, "devDependencies");

    let v = serde_json::to_string(&v)?;

    fs::write("package.json", v)?;

    Ok(())
}

fn patch_mode_cleanup() -> Result<(), Box<dyn Error>> {
    fs::copy("package.json.bkup", "package.json")?;
    fs::remove_file("package.json.bkup")?;

    Ok(())
}

pub fn run(config: &Config) -> Result<String, Box<dyn Error>> {
    if config.is_patch_mode {
        patch_mode_init()?;
    }

    let output = process::Command::new(NPM)
        .arg("outdated")
        .arg("--parseable")
        .output()
        .unwrap_or_else(|err| {
            // worst case scenario where they both fail just panic
            patch_mode_cleanup().unwrap();
            eprintln!("{}", err);
            process::exit(70)
        });

    if config.is_patch_mode {
        patch_mode_cleanup()?;
    }

    let output = String::from_utf8(output.stdout)?;

    Ok(output)
}

// Tests --------------------------------------------------------------

#[cfg(test)]
mod prefix_with_tilde_tests {
    use super::*;

    #[test]
    fn prefixes_strings() {
        assert_eq!(prefix_with_tilde("hello"), String::from("~hello"));
        assert_eq!(prefix_with_tilde("123456"), String::from("~123456"));
        assert_eq!(prefix_with_tilde("@something"), String::from("~@something"));
        assert_eq!(prefix_with_tilde(""), String::from("~"));
    }

    #[test]
    fn handles_empty() {
        assert_eq!(prefix_with_tilde(""), String::from("~"));
    }

    #[test]
    fn replaces_first_carat() {
        assert_eq!(prefix_with_tilde("^something"), String::from("~something"));
        assert_eq!(prefix_with_tilde("^@package"), String::from("~@package"));
        assert_eq!(
            prefix_with_tilde("^^fdjshafda"),
            String::from("~^fdjshafda")
        );
        assert_eq!(prefix_with_tilde("^1234"), String::from("~1234"));
    }
}

#[cfg(test)]
mod prefix_all_entries_with_tilde_tests {
    use super::*;

    fn test_input() -> Value {
        let example = r#"
        {
            "name": "John Doe",
            "age": 43,
            "dependencies": {
                "package": "^1.2.3",
                "@org/package": "^5.0.0",
                "p": "1.0.0"
            },
            "devDependencies": {
                "something": "^0.0.1",
                "@abc/tree": "^6.0.0",
                "blob": "1135.3.0"
            }
        }"#;

        serde_json::from_str(example).unwrap()
    }

    #[test]
    fn updates_as_expected_1() {
        let mut input = test_input();
        let expected = r#"
        {
            "name": "John Doe",
            "age": 43,
            "dependencies": {
                "package": "~1.2.3",
                "@org/package": "~5.0.0",
                "p": "~1.0.0"
            },
            "devDependencies": {
                "something": "^0.0.1",
                "@abc/tree": "^6.0.0",
                "blob": "1135.3.0"
            }
        }"#;
        let expected: Value = serde_json::from_str(expected).unwrap();

        prefix_all_entries_with_tilde(&mut input, "dependencies");

        assert_eq!(input, expected);
    }

    #[test]
    fn updates_as_expected_2() {
        let mut input = test_input();
        let expected = r#"
        {
            "name": "John Doe",
            "age": 43,
            "dependencies": {
                "package": "^1.2.3",
                "@org/package": "^5.0.0",
                "p": "1.0.0"
            },
            "devDependencies": {
                "something": "~0.0.1",
                "@abc/tree": "~6.0.0",
                "blob": "~1135.3.0"
            }
        }"#;
        let expected: Value = serde_json::from_str(expected).unwrap();

        prefix_all_entries_with_tilde(&mut input, "devDependencies");

        assert_eq!(input, expected);
    }

    #[test]
    fn updates_as_expected_3() {
        let mut input = test_input();
        let expected = r#"
        {
            "name": "John Doe",
            "age": 43,
            "dependencies": {
                "package": "^1.2.3",
                "@org/package": "^5.0.0",
                "p": "1.0.0"
            },
            "devDependencies": {
                "something": "^0.0.1",
                "@abc/tree": "^6.0.0",
                "blob": "1135.3.0"
            }
        }"#;
        let expected: Value = serde_json::from_str(expected).unwrap();

        prefix_all_entries_with_tilde(&mut input, "doesNotExist");

        assert_eq!(input, expected);
    }

    #[test]
    fn handles_no_dev_deps() {
        let mut input = serde_json::from_str(
            r#"
        {
            "name": "John Doe",
            "age": 43,
            "dependencies": {
                "package": "^1.2.3",
                "@org/package": "^5.0.0",
                "p": "1.0.0"
            }
        }"#,
        )
        .unwrap();
        let expected = r#"
        {
            "name": "John Doe",
            "age": 43,
            "dependencies": {
                "package": "^1.2.3",
                "@org/package": "^5.0.0",
                "p": "1.0.0"
            }
        }"#;
        let expected: Value = serde_json::from_str(expected).unwrap();

        prefix_all_entries_with_tilde(&mut input, "devDependencies");

        assert_eq!(input, expected);
    }

    #[test]
    fn handles_no_deps() {
        let mut input = serde_json::from_str(
            r#"
        {
            "name": "John Doe",
            "age": 43,
            "devDependencies": {
                "something": "^0.0.1",
                "@abc/tree": "^6.0.0",
                "blob": "1135.3.0"
            }
        }"#,
        )
        .unwrap();
        let expected = r#"
        {
            "name": "John Doe",
            "age": 43,
            "devDependencies": {
                "something": "^0.0.1",
                "@abc/tree": "^6.0.0",
                "blob": "1135.3.0"
            }
        }"#;
        let expected: Value = serde_json::from_str(expected).unwrap();

        prefix_all_entries_with_tilde(&mut input, "dependencies");

        assert_eq!(input, expected);
    }
}

#[cfg(test)]
mod patch_mode_init {
    use super::*;
    use serial_test::serial;
    use std::{env, path::Path};

    #[test]
    #[serial]
    fn patch_mode_init_works() {
        let current = env::current_dir().unwrap();

        env::set_current_dir("./src/test_files").unwrap();
        patch_mode_init().unwrap();

        assert_eq!(Path::new("./package.json").exists(), true);
        assert_eq!(Path::new("./package.json.bkup").exists(), true);

        fs::copy("package.json.bkup", "package.json").unwrap();
        fs::remove_file("package.json.bkup").unwrap();

        env::set_current_dir(current).unwrap();
    }
}

#[cfg(test)]
mod patch_mode_cleanup {
    use serial_test::serial;

    use super::*;
    use std::{env, path::Path};

    #[test]
    #[serial]
    fn cleanup_files() {
        let current = env::current_dir().unwrap();

        env::set_current_dir("./src/test_files").unwrap();
        fs::copy("package.json", "package.json.bkup").unwrap();

        patch_mode_cleanup().unwrap();

        assert_eq!(Path::new("./package.json").exists(), true);
        assert_eq!(Path::new("./package.json.bkup").exists(), false);

        env::set_current_dir(current).unwrap();
    }
}
