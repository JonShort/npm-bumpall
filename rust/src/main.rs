use std::process;

fn main() {
    let output = process::Command::new("npm")
        .arg("outdated")
        .output()
        .expect("Failed running npm script!");

    let output = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            process::exit(0x0100)
        }
    };

    println!("{}", output);

    let split_by_eol: Vec<&str> = output.split_terminator("\n").collect();
    let split_by_eol = &split_by_eol[1..];

    let packages: Vec<String> = split_by_eol
        .iter()
        .map(|s| {
            let word = get_first_word(s);
            format!("{}@latest", word)
        })
        .collect();

    let readable_command = format!("npm i {}", packages.join(" "));
    println!("Running:\n{}\n\n", readable_command);

    let mut install = process::Command::new("npm")
        .arg("i")
        .args(&packages)
        .spawn()
        .expect("Failed running npm script!");

    let status = install.wait().expect("npm script failed");

    if status.success() {
        println!("All packages now bumped to latest");
    } else {
        println!("Issue installing packages - try running manually");
    }
}

fn get_first_word(s: &str) -> &str {
    let idx = match s.find(" ") {
        Some(i) => i,
        _ => return "",
    };

    match s.get(..idx) {
        Some(word) => word,
        _ => return "",
    }
}
