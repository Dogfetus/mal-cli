use crossterm::event::KeyCode;
use ratatui::DefaultTerminal;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::{io, sync::mpsc};
use crate::controller::get_handlers;
use crate::ui;

// store the screens the user is at.
#[allow(dead_code)]
#[derive(Debug)]
pub enum CurrentScreen {
    Main,
    Anime,
    Manga,
    Info,
    Profile,
    Settings,
}

// here will all the details of a specific anime or manga be stored.
#[allow(dead_code)]
pub enum CurrentInfo {
    Anime,
    Manga,
}


#[allow(dead_code)]
pub enum Event {
    KeyPress(crossterm::event::KeyEvent),
    MouseClick(crossterm::event::MouseEvent),
}


#[allow(dead_code)]
pub struct App {
    pub key_input: String,              // the currently being edited json key.
    pub value_input: String,            // the currently being edited json value.

    pub current_screen: CurrentScreen,
    pub current_info: Option<CurrentInfo>,

    pub exit: bool,

    rx: mpsc::Receiver<Event>,
    sx: mpsc::Sender<Event>,
    threads: Vec<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl App {
    pub fn new() -> App {
        let (sx, rx) = mpsc::channel::<Event>();

        App {
            key_input: String::new(),
            value_input: String::new(),

            current_screen: CurrentScreen::Main,
            current_info: None,
            exit: false,

            rx,
            sx,
            threads: Vec::new(),
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // run any background threads
        self.spawn_background();

        // WARNING: don't use just unwrap here
        while !self.exit {
            terminal.draw( |frame| ui::draw(frame, self))?;
            match self.rx.recv().unwrap() {
                Event::KeyPress(key_event) => self.handle_input(key_event),           
                _ => {}
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, key_event: crossterm::event::KeyEvent) {
        if key_event.kind != crossterm::event::KeyEventKind::Press {
            return;
        }

        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('a') => self.current_screen = CurrentScreen::Anime,
            KeyCode::Char('m') => self.current_screen = CurrentScreen::Main,
            _ => { return }
        }
    }

    /// spawn the background threads (one for each handler)
    ///TODO: find a better way to stop the threads when the app exits.
    fn spawn_background(&mut self) {
        for handler in get_handlers() {
            let _sx = self.sx.clone();
            let _stop = self.stop.clone();
            let _thread = thread::spawn(move || {
                handler(_sx, _stop);
            });
            self.threads.push(_thread);
        }
    }

}



impl Drop for App {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);

        println!("Stopping threads...");
        for handle in self.threads.drain(..) {
            let _ = handle.join();
        }

        // sending a fake key input to stop the input handler (for now?)

    }
}

