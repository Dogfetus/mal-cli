use crate::handlers::get_handlers;
use crate::mal::MalClient;
use crate::mal::models::anime::Anime;
use crate::mal::models::anime::AnimeId;
use crate::mal::models::anime::DeleteOrUpdate;
use crate::mal::models::anime::MyListStatus;
use crate::player;
use crate::screens::BackgroundUpdate;
use crate::screens::ScreenManager;
use crate::screens::screens::*;
use crate::utils::store::Store;

use crossterm::event::KeyCode;
use image::DynamicImage;
use ratatui::prelude::CrosstermBackend;
use ratatui::DefaultTerminal;
use ratatui::Terminal;
use std::io;
use std::io::stdout;
use std::io::Stdout;
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
    PlayAnime(AnimeId),
    SwitchScreen(&'static str),
    ShowOverlay(AnimeId),
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

    fn play_anime(&mut self, anime: Anime) -> Option<()> {
        match self.anime_player.play_anime(&anime) {
            Ok(details) => {
                if details.percentage > 90 {
                    self.shared_info.anime_store.update(anime.id, |anime_to_update| {
                        if anime_to_update.my_list_status.num_episodes_watched < anime_to_update.num_episodes {
                            anime_to_update.my_list_status.num_episodes_watched += 1;
                            anime_to_update.my_list_status.status = "watching".to_string();
                        } else {
                            anime_to_update.my_list_status.status = "completed".to_string();
                        }
                    });

                    let updated = self.shared_info.anime_store.get(&anime.id)?;
                    self.shared_info.mal_client.update_user_list_async(updated);
                } 

                self.screen_manager.refresh();
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
                Action::Quit => {
                    self.is_running = false;
                }
                Action::ShowOverlay(anime_id) => {
                    self.screen_manager.toggle_overlay(anime_id);
                }
                Action::PlayAnime(anime_id) => {
                    let anime = self.shared_info.anime_store.get(&anime_id).unwrap_or_else(|| {
                        self.screen_manager.show_error("Unexpected anime given".to_string());
                        return Anime::default();
                    });
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
impl Drop for App {
    fn drop(&mut self) {
        // restore terminal view
        ratatui::restore();
    }
}

