use std::collections::HashMap;
use std::io;

use promkit::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    grapheme::Graphemes,
    handler, keybind, readline, select, Handler, Output, Result,
};
use radix_trie::Trie;

pub struct State {
    pub trie: Trie<Graphemes, usize>,
    pub readline: readline::State,
    pub select: select::State,
}

impl Output for State {
    type Output = String;

    fn output(&self) -> String {
        self.select.output()
    }
}

pub struct KeyBind(keybind::KeyBind<State>);

impl Default for KeyBind {
    fn default() -> Self {
        let mut kb = KeyBind(keybind::KeyBind::<State> {
            event_mapping: HashMap::default(),
            handle_input: Some(crate::handler::input_char()),
            handle_resize: Some(crate::handler::wrap_readline_handler(handler::reload::<
                Buffer,
                readline::state::With,
                io::Stdout,
            >())),
        });
        kb.0.assign(vec![
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                }),
                handler::enter::<State>(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                handler::interrupt::<State>(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                }),
                crate::handler::wrap_readline_handler(readline::handler::move_left()),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                }),
                crate::handler::wrap_readline_handler(readline::handler::move_right()),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                crate::handler::wrap_readline_handler(readline::handler::move_head()),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                crate::handler::wrap_readline_handler(readline::handler::move_tail()),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                }),
                crate::handler::erase_char(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                crate::handler::erase_all(),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                }),
                crate::handler::wrap_select_handler(select::handler::move_up()),
            ),
            (
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                }),
                crate::handler::wrap_select_handler(select::handler::move_down()),
            ),
        ]);
        kb
    }
}

impl Handler<State> for KeyBind {
    fn handle(&mut self, ev: Event, out: &mut io::Stdout, state: &mut State) -> Result<bool> {
        self.0.handle(ev, out, state)
    }
}
