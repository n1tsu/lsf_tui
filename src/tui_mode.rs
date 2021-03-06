use rand::prelude::*;

use std::io;
use std::time::{Duration, Instant};

// tui
use tui::backend::TermionBackend;
use tui::Terminal;

// termion
use termion::event::Key;
use termion::raw::IntoRawMode;

// local modules
use crate::draw::{draw_dictionary, draw_learn, WordState};
// use crate::draw::WORDS_LEARN_SIZE;
use crate::event::{Event, Events};
use crate::loader::{Categorie, Word};
use crate::selection::Selection;

// video search
use crate::search_video::{query_videos, play_video};

pub fn tui_routine(categories: Vec<Categorie>, _all_words: Vec<Word>) -> Result<(), io::Error> {
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
    let mut words_learn_set: Vec<(&Word, WordState)> = vec![];
    let mut help = false;
    let mut time = Duration::new(0, 0);

    loop {
        // Call update function and quit if it return 'Stop'
        match update(
            &events,
            &mut tab_index,
            &mut states,
            &categories,
            &mut help,
            &mut words_learn_set,
        ) {
            UpdateState::Stop => break,
            // Refresh the TUI widgets
            UpdateState::Continue => {
                if tab_index == 0 {
                    // Draw the dictionary mode
                    if swap == 1 {
                        states.reset();
                    }
                    draw_dictionary(&mut terminal, &mut states, &categories);
                    swap = 0;
                } else if tab_index == 1 {
                    // Reset variable because we swap tab
                    if swap == 0 {
                        begin = Instant::now();
                        let mut rng = rand::thread_rng();
                        // words_set = all_words.iter().choose_multiple(&mut rng, WORDS_LEARN_SIZE);
                        let cat_index = states.get_categorie_index();
                        let mut words_set =
                            categories[cat_index].words.iter().collect::<Vec<&Word>>();
                        words_set.shuffle(&mut rng);
                        words_learn_set = words_set
                            .iter()
                            .map(|&word| (word, WordState::Next))
                            .collect();

                        words_learn_set[0].1 = WordState::Current;

                        // Dirty hacks
                        states.reset_word_index();
                        states.focus_right(words_set.len());

                        help = false;
                    }
                    // Calculate time since swap
                    let now = Instant::now();
                    if !states.is_done() {
                        time = now.duration_since(begin);
                    }

                    // Draw the learn mode
                    draw_learn(
                        &mut terminal,
                        &mut words_learn_set,
                        &mut states,
                        &time,
                        &mut help,
                    );
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
    help: &mut bool,
    words_learn_set: &mut Vec<(&Word, WordState)>,
) -> UpdateState {
    // Try to receive an event, handle it if any, then just return
    if let Ok(x) = events.rx.recv() {
        // An event has been sent, let's handle it
        // If this event is an input, do some actions
        if let Event::Input(input) = x {
            if *tab_index == 0 {
                return input_tab_one(input, states, tab_index, categories);
            } else if *tab_index == 1 {
                return input_tab_two(input, states, help, tab_index, categories, words_learn_set);
            } else {
                panic!("Tab index is invalid !")
            }
        };
    }
    UpdateState::Continue
}

fn input_tab_one(
    input: Key,
    states: &mut Selection,
    tab_index: &mut usize,
    categories: &[Categorie],
) -> UpdateState {
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
        Key::Char('v') => {
            let word = &categories[states.get_categorie_index()].words[states.get_word_index()];
            let urls = query_videos(&word.name.to_string());
            if !urls.is_empty() {
                play_video(&urls[0]).unwrap();
            }
        }
        // Change tabs
        _ => {}
    };

    UpdateState::Continue
}

fn input_tab_two(
    input: Key,
    states: &mut Selection,
    help: &mut bool,
    tab_index: &mut usize,
    categories: &[Categorie],
    words_learn_set: &mut Vec<(&Word, WordState)>,
) -> UpdateState {
    match input {
        // Change tabs
        Key::Char('1') => {
            *tab_index = 0;
        }
        // Quit
        Key::Char('q') => return UpdateState::Stop,
        // Change word index in learn
        Key::Char('n') => {
            let cat_index = states.get_categorie_index();

            // Change word state
            if words_learn_set[states.get_word_index()].1 != WordState::Failed {
                words_learn_set[states.get_word_index()].1 = WordState::Valided;
            }

            // If the index is over total words in categorie
            if states.get_word_index() < categories[cat_index].words.len() - 1 {
                states.down();
                *help = false;
                words_learn_set[states.get_word_index()].1 = WordState::Current;
            } else {
                states.set_done()
            }
        }
        // Display help in learn
        Key::Char('h') => {
            // Change word state
            words_learn_set[states.get_word_index()].1 = WordState::Failed;
            *help = !*help;
        }
        _ => {}
    };

    UpdateState::Continue
}
