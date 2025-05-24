use crossterm::event::KeyCode;
use ratatui::DefaultTerminal;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::{io, sync::mpsc};
use crate::handlers::get_handlers;
use crate::ui::ScreenManager;
use crate::ui::screens::*;

pub enum Action {
    SwitchScreen(&'static str),
    Quit,
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
    Resize(u16, u16), 
    Rerender
}


#[allow(dead_code)]
pub struct App {
    pub screen_manager: ScreenManager,
    pub current_info: Option<CurrentInfo>,
    pub is_running: bool,
    pub sx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    threads: Vec<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl App {
    pub fn new() -> App {
        let (sx, rx) = mpsc::channel::<Event>();

        App {
            screen_manager: ScreenManager::new(sx.clone()),
            current_info: None,
            is_running: true,

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
        while self.is_running {
            terminal.draw( |frame| self.screen_manager.draw(frame))?;
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

        let result = self.screen_manager.handle_input(key_event);
        if let Some(action) = result {
            match action {
                Action::SwitchScreen(screen_name) => {
                    self.screen_manager.change_screen(screen_name);
                }
                Action::Quit => {
                    self.is_running = false;
                }
            }
        }

        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.is_running = false
            },
            KeyCode::Char('a') => self.screen_manager.change_screen(INFO),
            KeyCode::Char('m') => self.screen_manager.change_screen(LOGIN),
            _ => { return }
        }
    }

    /// spawn the background threads (one for each handler)
    ///TODO: find a better way to stop the threads when the app exits
    // TODO: the keyhandler thread waits for input after stopping the app  (bad)
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

