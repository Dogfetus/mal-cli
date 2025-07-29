use crate::handlers::get_handlers;
use crate::mal::MalClient;
use crate::mal::models::anime::Anime;
use crate::mal::models::anime::AnimeId;
use crate::mal::models::anime::DeleteOrUpdate;
use crate::mal::models::anime::MyListStatus;
use crate::screens::BackgroundUpdate;
use crate::screens::ScreenManager;
use crate::screens::screens::*;
use crate::utils::store::Store;

use crossterm::event::KeyCode;
use image::DynamicImage;
use ratatui::DefaultTerminal;
use regex::Regex;
use std::io;
use std::io::ErrorKind;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub struct ExtraInfo {
    pub app_sx: mpsc::Sender<Event>,
    pub mal_client: Arc<MalClient>,
    pub anime_store: Store<Anime>,
}

pub enum Action {
    PlayAnime(Anime),
    SwitchScreen(&'static str),
    ShowOverlay(Anime),
    NavbarSelect(bool),
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
    StorageUpdate(AnimeId, DeleteOrUpdate),
    Rerender,
}

#[allow(dead_code)]
pub struct App {
    mal_client: Arc<MalClient>,
    screen_manager: ScreenManager,
    current_info: Option<CurrentInfo>,
    is_running: bool,
    terminal: DefaultTerminal,
    shared_info: ExtraInfo,

    sx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    threads: Vec<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    ansi_regex: Regex,
}

impl App {
    pub fn new() -> App {
        let (sx, rx) = mpsc::channel::<Event>();
        let mal_client = Arc::new(MalClient::new());
        let terminal = ratatui::init();
        let universal_info = ExtraInfo {
            app_sx: sx.clone(),
            mal_client: mal_client.clone(),
            anime_store: Store::new(),
        };

        App {
            mal_client: mal_client.clone(),
            screen_manager: ScreenManager::new(universal_info.clone()),
            current_info: None,
            is_running: true,
            terminal,
            shared_info: universal_info,

            rx,
            sx,
            threads: Vec::new(),
            stop: Arc::new(AtomicBool::new(false)),
            ansi_regex: Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\([AB]|\r|\x1b[78]").unwrap(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // run any background threads
        self.spawn_background();

        // WARNING: don't use just unwrap here
        while self.is_running {
            self.terminal
                .draw(|frame| self.screen_manager.render_screen(frame))?;

            let first_event = self.rx.recv().unwrap();
            let mut events = vec![first_event];

            // in case multiple events happen at the same time, we want to process them all
            while let Ok(event) = self.rx.try_recv() {
                events.push(event);
            }

            for event in events {
                match event {
                    Event::KeyPress(key_event) => self.handle_input(key_event),
                    Event::BackgroundNotice(mut update) => {
                        if let Some(animes) = update.take::<Vec<Anime>>("animes") {
                            self.shared_info.anime_store.add_bulk(animes);
                        }

                        self.screen_manager.update_screen(update);
                    }
                    Event::StorageUpdate(id, update) => match update {
                        DeleteOrUpdate::Deleted(_vec) => {
                            self.shared_info.anime_store.update(id, |anime| {
                                anime.my_list_status = MyListStatus::default();
                            });
                        }
                        DeleteOrUpdate::Updated(myliststatus) => {
                            self.shared_info.anime_store.update(id, |anime| {
                                anime.my_list_status = myliststatus;
                            });
                        }
                    },
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub fn play_anime(&mut self, anime: Anime) {
        if anime.status == "upcoming" {
            self.screen_manager.show_error(format!(
                "\"{}\"\nis not yet released.",
                if anime.alternative_titles.en.is_empty() {
                    anime.title
                } else {
                    anime.alternative_titles.en
                }
            ));
            return;
        }

        ratatui::restore();

        let next_episode = std::cmp::min(
            anime.my_list_status.num_episodes_watched + 1,
            anime.num_episodes,
        );

        let output = Command::new("ani-cli")
            .arg("--no-detach")
            .arg("--exit-after-play")
            .arg("-e")
            .arg(&next_episode.to_string())
            .arg(&anime.title)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                let messy_stdout = String::from_utf8_lossy(&output.stdout);
                let messy_stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = self.ansi_regex.replace_all(&messy_stdout, "").to_string();
                let stderr = self.ansi_regex.replace_all(&messy_stderr, "").to_string();
                println!("ani-cli output:\n{}", stdout);
                println!("ani-cli error:\n{}", stderr);
                println!("ani-cli t {:?}, {}", output, anime.title);
                if !stderr.is_empty() {
                    if stderr.contains("No results found!") {
                        self.screen_manager.show_error(format!(
                            "ani-cli replied:\nError: {}\nthe anime might not be available yet",
                            stderr
                        ));
                    } else {
                        self.screen_manager.show_error(format!(
                            "ani-cli replied:\nError: {}Exit code: {}\nOutput: {}",
                            stderr,
                            output.status.code().unwrap_or(-1),
                            stdout
                        ));
                    }
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    self.screen_manager
                        .show_error(format!("Can't seem to find ani-cli: \n{}", e));
                } else {
                    self.screen_manager
                        .show_error(format!("Error running ani-cli: \n{}", e));
                }
            }
        }

        self.terminal = ratatui::init();
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
                Action::ShowOverlay(anime) => {
                    self.screen_manager.toggle_overlay(anime);
                }
                Action::PlayAnime(anime) => {
                    self.play_anime(anime);
                }
                Action::NavbarSelect(selected) => {
                    self.screen_manager.toggle_navbar(selected);
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
