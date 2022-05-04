use std::cell::RefCell;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use clap::{Arg, Command};

use libc::{ioctl, TIOCSTI};
use promkit::{
    self, build::Builder, crossterm::style, grapheme::Graphemes, readline, select, state::Render,
    termutil, Prompt,
};
use radix_trie::Trie;

mod handler;
mod keybind;

use crate::keybind::{KeyBind, State};

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

fn main() -> promkit::Result<()> {
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

    let mut trie = Trie::<Graphemes, usize>::new();
    let file = File::open(path)?;
    let mut buf = Vec::<String>::new();
    for line in io::BufReader::new(file).lines().flatten() {
        let trimed = line.trim();
        buf.push(trimed.to_string());
        if !line.ends_with('\\') {
            trie.insert(Graphemes::from(buf.join("").replace('\\', "")), 0);
            buf.clear();
        }
    }

    let readline = readline::Builder::default().num_lines(1);
    let select = select::Builder::default()
        .selected_color(style::Color::DarkBlue)
        .init_move_down_lines(1)
        .suffix_after_trim("...");
    let mut hstr = Prompt::<State> {
        out: io::stdout(),
        handler: Rc::new(RefCell::new(KeyBind::default())),
        pre_run: Some(Box::new(
            |out: &mut io::Stdout, state: &mut State| -> promkit::Result<()> {
                state.readline.render(out)?;
                termutil::hide_cursor(out)?;
                state.select.render(out)?;
                termutil::show_cursor(out)
            },
        )),
        initialize: Some(Box::new(
            |out: &mut io::Stdout, state: &mut State| -> promkit::Result<()> {
                handler::finalize(state)?;
                state.readline.pre_render(out)
            },
        )),
        finalize: Some(Box::new(
            |out: &mut io::Stdout, _: &mut State| -> promkit::Result<()> { termutil::clear(out) },
        )),
        state: Box::new(State {
            trie,
            readline: *readline.state()?,
            select: *select.state()?,
        }),
    };
    let (line, exit_code) = hstr.run()?;
    if exit_code == 0 {
        fake_input(&line);
    }
    Ok(())
}
