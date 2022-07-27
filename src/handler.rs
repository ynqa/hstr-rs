use std::io;

use promkit::{readline, register::Register, select, selectbox::SelectBox, EventHandleFn, Result};
use radix_trie::TrieCommon;

use crate::State;

pub fn wrap_readline_handler(f: Box<EventHandleFn<readline::State>>) -> Box<EventHandleFn<State>> {
    Box::new(
        move |shape: Option<(u16, u16)>,
              input: Option<char>,
              out: &mut io::Stdout,
              state: &mut State| { f(shape, input, out, &mut state.readline) },
    )
}

pub fn wrap_select_handler(f: Box<EventHandleFn<select::State>>) -> Box<EventHandleFn<State>> {
    Box::new(
        move |shape: Option<(u16, u16)>,
              input: Option<char>,
              out: &mut io::Stdout,
              state: &mut State| { f(shape, input, out, &mut state.select) },
    )
}

/// Insert a char.
pub fn input_char() -> Box<EventHandleFn<State>> {
    Box::new(
        |_, input: Option<char>, out: &mut io::Stdout, state: &mut State| {
            readline::handler::input_char()(None, input, out, &mut state.readline)?;
            finalize(state)
        },
    )
}

/// Erase a char.
pub fn erase_char() -> Box<EventHandleFn<State>> {
    Box::new(
        |_, input: Option<char>, out: &mut io::Stdout, state: &mut State| {
            readline::handler::erase_char()(None, input, out, &mut state.readline)?;
            finalize(state)
        },
    )
}

/// Erase all chars.
pub fn erase_all() -> Box<EventHandleFn<State>> {
    Box::new(
        |_, input: Option<char>, out: &mut io::Stdout, state: &mut State| {
            readline::handler::erase_all()(None, input, out, &mut state.readline)?;
            finalize(state)
        },
    )
}

pub fn finalize(state: &mut State) -> Result<bool> {
    let query = &state.readline.0.editor.data;
    state.select.1.selected_cursor_position = 0;
    state.select.0.editor = match state.trie.get_raw_descendant(query) {
        Some(subtrie) => subtrie
            .iter()
            .fold(Box::new(SelectBox::default()), |mut sb, item| {
                sb.register(item.0.to_string());
                sb
            }),
        None => Box::new(SelectBox::default()),
    };
    Ok(false)
}
