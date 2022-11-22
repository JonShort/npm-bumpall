use std::env;

pub fn get_var(v: &str, fb: &str) -> String {
    match env::var(v) {
        Ok(e) => e,
        Err(_) => String::from(fb),
    }
}
