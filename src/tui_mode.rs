use rand::prelude::*;

use std::io;
use std::time::Instant;

// tui
use tui::backend::TermionBackend;
use tui::Terminal;

// termion
use termion::event::Key;
use termion::raw::IntoRawMode;

// local modules
use crate::draw::{draw_dictionary, draw_learn, WORDS_LEARN_SIZE};
use crate::event::{Event, Events};
use crate::loader::{Categorie, Word};
use crate::selection::Selection;

pub fn tui_routine(categories: Vec<Categorie>, all_words: Vec<Word>) -> Result<(), io::Error> {
    // Initialize terminal
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    // Spawn new threads for events
    // Get a 'Tick' information every 200ms and get inputs
    let events = Events::new(200);

    // Initialize current selection and current tab variable
    let mut states = Selection::new(categories.len());
    let mut tab_index = 0;

    // Variable to determine if we just swapped between tabs
    // If swap is 0 we were on tab 0 on last loop and if swap
    // is 1 we were on tab 1 on last loop.
    let mut swap = 0;
    let mut begin = Instant::now();
    let mut words_set = vec![];
    let mut word_index = 0;
    let mut help = false;

    loop {
        // Call update function and quit if it return 'Stop'
        match update(
            &events,
            &mut tab_index,
            &mut states,
            &categories,
            &mut word_index,
            &mut help,
        ) {
            UpdateState::Stop => break,
            // Refresh the TUI widgets
            UpdateState::Continue => {
                if tab_index == 0 {
                    // Draw the dictionary mode
                    draw_dictionary(&mut terminal, &mut states, &categories);
                    swap = 0;
                } else if tab_index == 1 {
                    // Reset variable because we swap tab
                    if swap == 0 {
                        begin = Instant::now();
                        let mut rng = rand::thread_rng();
                        words_set = all_words.iter().choose_multiple(&mut rng, WORDS_LEARN_SIZE);
                        word_index = 0;
                        help = false;
                    }
                    // Calculate time since swap
                    let now = Instant::now();
                    let time = now.duration_since(begin);

                    // Draw the learn mode
                    draw_learn(&mut terminal, &words_set, &time, &word_index, &mut help);
                    swap = 1;
                }
            }
        }
    }
    terminal.clear()?;
    Ok(())
}

enum UpdateState {
    Stop,
    Continue,
}

fn update(
    events: &Events,
    tab_index: &mut usize,
    states: &mut Selection,
    categories: &[Categorie],
    word_index: &mut usize,
    help: &mut bool,
) -> UpdateState {
    // Try to receive an event, handle it if any, then just return
    if let Ok(x) = events.rx.recv() {
        // An event has been sent, let's handle it
        // If this event is an input, do some actions
        if let Event::Input(input) = x {
            if *tab_index == 0 {
                return input_tab_one(input, states, tab_index, categories)
            } else if *tab_index == 1 {
                return input_tab_two(input, help, tab_index, word_index, categories)
            } else {
                panic!("Tab index is invalid !")
            }
        };
    }
    UpdateState::Continue
}

fn input_tab_one(input: Key, states: &mut Selection, tab_index: &mut usize, categories: &[Categorie]) -> UpdateState {
    match input {
        // Change tabs
        Key::Char('2') => {
            *tab_index = 1;
        }
        // Quit
        Key::Char('q') => return UpdateState::Stop,
        // Move selection
        Key::Char('j') => {
            states.down();
        }
        Key::Char('k') => {
            states.up();
        }
        Key::Char('h') => {
            states.focus_left();
        }
        Key::Char('l') => {
            states.focus_right(categories[states.get_categorie_index()].words.len());
        }
        // Change tabs
        _ => {}
    };

    UpdateState::Continue
}

fn input_tab_two(input: Key, help: &mut bool, tab_index: &mut usize, word_index: &mut usize, categories: &[Categorie]) -> UpdateState {
    match input {
        // Change tabs
        Key::Char('1') => {
            *tab_index = 0;
        }
        // Quit
        Key::Char('q') => return UpdateState::Stop,
        // Change word index in learn
        Key::Char('n') => {
            if *word_index < WORDS_LEARN_SIZE - 1 {
                *word_index += 1;
                *help = false;
            }
        }
        // Display help in learn
        Key::Char('h') => {
            *help = !*help;
        }
        _ => {}
    };

    UpdateState::Continue
}
