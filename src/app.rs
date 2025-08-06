use crate::mal::models::anime::AnimeId;
use crate::screens::BackgroundUpdate;
use crate::mal::models::anime::Anime;
use crate::handlers::get_handlers;
use crate::screens::ScreenManager;
use crate::screens::screens::*;
use crate::utils::store::Store;
use crate::mal::MalClient;
use crate::player;

use std::sync::atomic::AtomicBool;
use crossterm::event::KeyCode;
use ratatui::DefaultTerminal;
use std::thread::JoinHandle;
use std::fs::OpenOptions;
use image::DynamicImage;
use std::path::PathBuf;
use chrono::DateTime;
use std::sync::mpsc;
use std::io::Write;
use std::sync::Arc;
use chrono::Local;
use std::thread;
use std::io;

#[derive(Debug, Clone)]
pub struct ExtraInfo {
    pub app_sx: mpsc::Sender<Event>,
    pub mal_client: Arc<MalClient>,
    pub anime_store: Store<Anime>,
}

pub enum Action {
    PlayAnime(AnimeId),
    SwitchScreen(&'static str),
    ShowOverlay(AnimeId),
    NavbarSelect(bool),
    ShowError(String),
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
    StorageUpdate(AnimeId, Box<dyn FnOnce(&mut Anime) + Send>),
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
    anime_player: player::AnimePlayer,

    sx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    threads: Vec<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl App {
    pub fn new(terminal: DefaultTerminal) -> App {
        let (sx, rx) = mpsc::channel::<Event>();
        let mal_client = Arc::new(MalClient::new());
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
            anime_player: player::AnimePlayer::new(),

            rx,
            sx,
            threads: Vec::new(),
            stop: Arc::new(AtomicBool::new(false)),
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
                    Event::StorageUpdate(anime, updater ) => {
                        self.shared_info.anime_store.update(anime, |anime_to_update| {
                            updater(anime_to_update);
                        });
                        self.screen_manager.refresh();
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn logg_watched_info(&self, anime: &Anime, details: &player::PlayResult) {
        let app_dir = std::env::var("HOME").ok()
            .map(|home| PathBuf::from(home)
            .join(".local/share/mal-cli"))
            .expect("Failed to get app directory");

        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S");
        let log_file = app_dir.join("watch_history");
        let log_entry = format!(
            "{} -> {} -> \"{}\" -> {} -> {}/{} -> {} -> {}\n",
            timestamp,
            anime.id,
            anime.title,
            details.episode,
            details.current_time,
            details.total_time,
            details.percentage,
            details.completed
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("Failed to open log file");
    
        file.write_all(log_entry.as_bytes()).ok();
    }

    fn play_anime(&mut self, anime_id: AnimeId) -> Option<()> {
        let anime = self.shared_info.anime_store.get(&anime_id)?;

        match self.anime_player.play_anime(&anime) {
            Ok(details) => {

                // update teh status to now watching
                self.shared_info.anime_store.update(anime.id, |anime_to_update| {
                    anime_to_update.my_list_status.status = "watching".to_string();
                });

                if details.completed {
                    // update the store <-
                    self.shared_info.anime_store.update(anime.id, |anime_to_update| {
                        if anime_to_update.my_list_status.num_episodes_watched < anime_to_update.num_episodes {
                            anime_to_update.my_list_status.num_episodes_watched += 1;
                        } else {
                            anime_to_update.my_list_status.status = "completed".to_string();
                        }
                    });
                } 
                // get the anime again to make sure the details are up to date with the update above
                let updated = self.shared_info.anime_store.get(&anime.id)?;
                self.shared_info.mal_client.update_user_list_async(updated);
                self.screen_manager.refresh();
                self.logg_watched_info(&anime, &details);
            }
            Err(e) => {
                self.screen_manager.show_error(e.to_string());
            }
        }

        self.terminal = ratatui::init();
        None
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
                Action::ShowOverlay(anime_id) => {
                    self.screen_manager.toggle_overlay(anime_id);
                }
                Action::NavbarSelect(selected) => {
                    self.screen_manager.toggle_navbar(selected);
                }
                Action::PlayAnime(anime_id) => {
                    self.play_anime(anime_id);
                }
                Action::ShowError(message) => {
                    self.screen_manager.show_error(message);
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

    fn spawn_background(&mut self) {
        for handler in get_handlers() {
            let _sx = self.sx.clone();
            let _thread = thread::spawn(move || {
                handler(_sx);
            });
            self.threads.push(_thread);
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // restore terminal view
        ratatui::restore();
    }
}

