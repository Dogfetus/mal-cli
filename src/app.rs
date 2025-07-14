use crate::{
    handlers::get_handlers,
    mal::MalClient,
    screens::{BackgroundUpdate, ScreenManager, screens::*},
};
use crossterm::event::KeyCode;
use image::DynamicImage;
use ratatui::DefaultTerminal;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::{self, JoinHandle};
use std::{io, sync::mpsc};

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
    BackgroundNotice(BackgroundUpdate),
    ImageCached(usize, DynamicImage),
    Rerender,
}

#[allow(dead_code)]
pub struct App {
    mal_client: Arc<MalClient>,
    screen_manager: ScreenManager,
    current_info: Option<CurrentInfo>,
    is_running: bool,

    sx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    threads: Vec<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl App {
    pub fn new() -> App {
        let (sx, rx) = mpsc::channel::<Event>();
        let mal_client = Arc::new(MalClient::new());

        App {
            mal_client: mal_client.clone(),
            screen_manager: ScreenManager::new(sx.clone(), mal_client),
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
            terminal.draw(|frame| self.screen_manager.render_screen(frame))?;

            let first_event = self.rx.recv().unwrap();
            let mut events = vec![first_event];

            // in case multiple events happen at the same time, we want to process them all
            while let Ok(event) = self.rx.try_recv() {
                events.push(event);
            }

            for event in events {
                match event {
                    Event::KeyPress(key_event) => self.handle_input(key_event),
                    Event::BackgroundNotice(update) => {
                        self.screen_manager.update_screen(update);
                    }
                    _ => {}
                }
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
        if key_event
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            match key_event.code {
                KeyCode::Char('c') => self.is_running = false,
                KeyCode::Char('f') => self.screen_manager.change_screen(SEARCH),
                KeyCode::Char('o') => self.screen_manager.change_screen(OVERVIEW),
                KeyCode::Char('s') => self.screen_manager.change_screen(SEASONS),
                KeyCode::Char('i') => self.screen_manager.change_screen(LIST),
                KeyCode::Char('p') => self.screen_manager.change_screen(PROFILE),
                _ => return,
            }
        }
    }

    // TODO: might not even need the stop since the thread will exit when the app exits
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

// TODO: check if this is even necessary? ii mean its rust right
//
// impl Drop for App {
//     fn drop(&mut self) {
//         self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
//
//         println!("Stopping threads...");
//         for handle in self.threads.drain(..) {
//             let _ = handle.join();
//         }
//
//         // sending a fake key input to stop the input handler (for now?)
//
//     }
// }
//
