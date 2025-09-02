use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use super::widgets::animebox::AnimeBox;
use super::widgets::navigatable::Navigatable;
use super::{BackgroundUpdate, ExtraInfo, Screen};
use crate::add_screen_caching;
use crate::app::{Action, Event};
use crate::config::{HIGHLIGHT_COLOR, PRIMARY_COLOR};
use crate::mal::models::anime::AnimeId;
use crate::utils::functionStreaming::StreamableRunner;
use crate::utils::get_app_dir;
use crate::utils::imageManager::ImageManager;
use crossterm::event::{KeyCode, KeyEvent};
use indexmap::IndexSet;
use ratatui::layout::{Margin, Rect};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear},
};
use tui_widgets::big_text::{BigText, PixelSize};


#[derive(PartialEq, Clone)]
enum Focus {
    NavBar,
    Content,
}

#[derive(Clone)]
struct List {
    title: String,
    navigatable: Navigatable,
    items: Vec<AnimeId>,
}

#[derive(Clone)]
pub struct OverviewScreen {
    bg_loaded: bool,
    app_info: ExtraInfo,
    image_manager: Arc<Mutex<ImageManager>>,

    navigation: Navigatable,
    lists: Vec<List>,
    focus: Focus,
}

impl OverviewScreen {
    pub fn new(info: ExtraInfo) -> Self {
        Self {
            app_info: info,
            bg_loaded: false,
            image_manager: Arc::new(Mutex::new(ImageManager::new())),
            navigation: Navigatable::new((3, 1)),
            lists: vec![
                List {
                    title: "Recently Watched".to_string(),
                    navigatable: Navigatable::new((1, 5)),
                    items: vec![],
                },
                List {
                    title: "Suggested Animes".to_string(),
                    navigatable: Navigatable::new((1, 5)),
                    items: vec![],
                },
                List {
                    title: "Most Popular".to_string(),
                    navigatable: Navigatable::new((1, 5)),
                    items: vec![],
                },
            ],
            focus: Focus::Content,
        }
    }
}

impl Screen for OverviewScreen {
    add_screen_caching!();

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let [_, content] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), 
                Constraint::Fill(1)
            ])
            .areas(area);

        // this is the outer navigatable meaning it splits into the vertical thre sections
        self.navigation
            .construct_mut(&mut self.lists, content, |list, area, highlighted| {
                let area = Rect::new(
                    area.x,
                    area.y + 3,
                    area.width,
                    area.height.saturating_sub(2),
                );

                // determine the highlighted color
                let color = if highlighted && self.focus == Focus::Content {
                    HIGHLIGHT_COLOR
                } else {
                    PRIMARY_COLOR
                };

                // draw a box for the highlighted section
                let block = Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(color));
                frame.render_widget(block, area.inner(Margin {
                    vertical: 1,
                    horizontal: 3,
                }));

                // split into title and list area (for each list section)
                let [title_area, list_area, _] = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), 
                        Constraint::Fill(1), 
                        Constraint::Length(1)
                    ])
                    .areas(area.inner(Margin {
                        vertical: 0,
                        horizontal: 8,
                    }));

                // add margin to the title 
                let [_, title_area] = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Fill(1), Constraint::Length(3)])
                    .areas(title_area);

                let title = BigText::builder()
                    .style(Style::new().fg(color))
                    .pixel_size(PixelSize::Sextant)
                    .lines(vec![list.title.clone().into()])
                    .build();

                frame.render_widget(title, title_area);


                // lett the user know nothin gis there yet if the list is empty
                if list.items.is_empty() {
                    let text = "Nothing here yet!";
                    let paragraph = Paragraph::new(text)
                        .style(Style::default().fg(Color::Red))
                        .wrap(Wrap { trim: true });
                    frame.render_widget(paragraph, list_area);
                    return;
                }

                // this is the inner navigatable (the vertical sections)
                list.navigatable.construct(
                    &list.items,
                    list_area,
                    |anime_id, inner_area, inner_highlighted| {
                        if let Some(anime) = &self.app_info.anime_store.get(anime_id) {
                            AnimeBox::render(
                                anime,
                                &self.image_manager,
                                frame,
                                inner_area.inner(Margin {
                                    vertical: 0,
                                    horizontal: 3,
                                }),
                                inner_highlighted && highlighted && self.focus == Focus::Content,
                            )
                        } 
                    },
                );
            });
    }

    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match self.focus {
            Focus::NavBar => {
                self.focus = Focus::Content;
            }

            Focus::Content => {
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Up => {
                            self.focus = Focus::NavBar;
                            return Some(Action::NavbarSelect(true));
                        }
                        _ => {}
                    }

                    return None;
                }

                match key_event.code {
                    KeyCode::Char('k') | KeyCode::Down => {
                        self.navigation.move_down();
                    }
                    KeyCode::Char('j') | KeyCode::Up => {
                        self.navigation.move_up();
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if let Some(selected) = self.navigation.get_selected_item_mut(&mut self.lists) {
                            selected.navigatable.move_right();
                        }
                    }

                    KeyCode::Char('h') | KeyCode::Left => {
                        if let Some(selected) = self.navigation.get_selected_item_mut(&mut self.lists) {
                            selected.navigatable.move_left();
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = self.navigation.get_selected_item_mut(&mut self.lists) {
                            if let Some(anime_id) = selected.navigatable.get_selected_item(&selected.items) {
                                return Some(Action::ShowOverlay(*anime_id));
                            }
                        }
                    }
                    _ => {}
                }

            }
        }

        None
    }

    fn background(&mut self) -> Option<JoinHandle<()>> {
        let already_loaded = self.bg_loaded;
        ImageManager::init_with_threads(&self.image_manager, self.app_info.app_sx.clone());
        let info = self.app_info.clone();
        let id = self.get_name();
        let sender = info.app_sx.clone();
        let app_dir = get_app_dir();
        let log_file = app_dir.join("watch_history");

        Some(thread::spawn(move || {
            let anime_generator = StreamableRunner::new()
                .stop_at(1);

            let mut cached_ids: Vec<AnimeId> = Vec::new();

            if let Ok(file) = OpenOptions::new().read(true).open(log_file) {
                // then we fetch the animes data from the mal api (this is just the users list as
                // the watchd animes will allways be in the users list after a watch)
                // this information will just be handled by the app and the store, and will not be
                // retrieved in this local apply_update

                // this is the users list of animes
                for animes in anime_generator.run(|offset, limit| {
                    info.mal_client
                        .get_anime_list(None, offset, limit)
                }) {
                    cached_ids.extend(animes.iter().map(|a| a.id.clone()));
                    let update = BackgroundUpdate::new(id.clone())
                        .set("animes", animes);
                    info.app_sx.send(Event::BackgroundNotice(update)).ok();
                }

                // this is first to fetch the file where the recent watched animes are
                let content = BufReader::new(file);
                let entries: Vec<String> = content.lines().filter_map(|line| line.ok()).collect();
                let mut animes = IndexSet::new();

                for entry in entries.iter().rev() {
                    let parts: Vec<&str> = entry.split(" -> ").collect();
                    if parts.len() < 7 {
                        // unexpected format, skip this entry
                        continue;
                    }

                    // idk what to do with this inforamiton yet but here it is.
                    let (timestamp, anime_id, title, episode, watched_time, percentage, completed) = (
                        parts[0].to_string(),
                        parts[1].parse::<AnimeId>().expect("Failed to read history"),
                        parts[2].to_string(),
                        parts[3].to_string(),
                        parts[4].to_string(),
                        parts[5].to_string(),
                        parts[6].to_string(),
                    );

                    // save each individual anime id to the set
                    if !cached_ids.contains(&anime_id) {
                        // if the anime is not in the users list, skip it cus they removed it
                        continue;
                    }
                    animes.insert(anime_id);
                }

                let animes: Vec<AnimeId> = animes.into_iter().collect();
                let update = BackgroundUpdate::new(id.clone()).set("WatchHistory", animes);
                sender.send(Event::BackgroundNotice(update)).ok();

                if already_loaded {
                    return;
                }

            }

            // this is the suggested animes
            for animes in anime_generator.run(|offset, limit| {
                info.mal_client
                    .get_suggested_anime(offset, limit)
            }) {
                let anime_ids = animes.iter().map(|a| a.id.clone()).collect::<Vec<_>>();
                let update = BackgroundUpdate::new(id.clone())
                    .set("animes", animes)
                    .set("SuggestedAnime", anime_ids);
                info.app_sx.send(Event::BackgroundNotice(update)).ok();
            }

            // this is the most popular animes
            for animes in anime_generator.run(|offset, limit| {
                info.mal_client
                    .get_top_anime("bypopularity".to_string(), offset, limit)
            }) {
                let anime_ids = animes.iter().map(|a| a.id.clone()).collect::<Vec<_>>();
                let update = BackgroundUpdate::new(id.clone())
                    .set("animes", animes)
                    .set("PopularAnime", anime_ids);
                info.app_sx.send(Event::BackgroundNotice(update)).ok();
            }
        }))
    }

    fn apply_update(&mut self, mut update: BackgroundUpdate) {
        if let Some(watch_history) = update.take::<Vec<AnimeId>>("WatchHistory") {
            self.lists[0].items = watch_history;
        }

        if let Some(suggested_anime) = update.take::<Vec<AnimeId>>("SuggestedAnime") {
            self.lists[1].items = suggested_anime;
        }

        if let Some(popular_anime) = update.take::<Vec<AnimeId>>("PopularAnime") {
            self.lists[2].items = popular_anime;
        }
    }
}
