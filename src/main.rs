use std::{
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use clap::{Arg, Command};
use libc::{ioctl, TIOCSTI};
use promkit::{self, preset::QuerySelect};
use radix_trie::{Trie, TrieCommon};

fn fake_input(s: &str) {
    for byte in s.as_bytes() {
        unsafe {
            ioctl(0, TIOCSTI, byte);
        }
    }
}

fn path_exists(path: &str) -> Result<(), String> {
    match Path::new(path).exists() {
        true => Ok(()),
        false => Err(format!("no such file: {}", path)),
    }
}

fn home_dir() -> String {
    dirs::home_dir()
        .expect("cannot get home dir path")
        .to_str()
        .expect("home dir path include non UTF-8 strings")
        .to_string()
}

fn main() -> promkit::error::Result {
    let zsh_history = home_dir() + "/.zsh_history";

    let histroy_path = Arg::new("path")
        .short('p')
        .long("path")
        .help("Path of the history file")
        .required(false)
        .takes_value(true)
        .allow_invalid_utf8(true)
        .default_value(&zsh_history)
        .validator(path_exists);

    let matches = Command::new("hstr")
        .version("0.1.0")
        .arg(histroy_path)
        .get_matches();

    let path = matches.value_of_os("path").map(PathBuf::from).unwrap();

    let mut trie = Trie::<String, usize>::new();
    let file = File::open(path)?;
    let mut buf = Vec::<String>::new();
    for line in io::BufReader::new(file).lines().flatten() {
        let trimed = line.trim();
        buf.push(trimed.to_string());
        if !line.ends_with('\\') {
            trie.insert(buf.join("").replace('\\', ""), 0);
            buf.clear();
        }
    }

    let mut hstr = QuerySelect::new(
        trie.clone().iter().map(|item| item.0),
        move |text, items| -> Vec<String> {
            match trie.get_raw_descendant(text) {
                Some(subtrie) => subtrie.iter().map(|item| item.0.clone()).collect(),
                None => items.clone(),
            }
        },
    )
    .prompt()?;

    fake_input(&hstr.run()?);
    Ok(())
}
