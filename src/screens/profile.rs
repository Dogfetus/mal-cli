use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;

use crate::app::Action;
use crate::app::Event;
use crate::config::HIGHLIGHT_COLOR;
use crate::config::PRIMARY_COLOR;
use crate::mal::models::anime::FavoriteAnime;
use crate::mal::models::user::User;
use crate::utils::imageManager::ImageManager;
use crate::utils::terminalCapabilities::TERMINAL_RATIO;

use super::BackgroundUpdate;
use super::ExtraInfo;
use super::Screen;
use super::screens::*;
use super::widgets::navbar::NavBar;
use super::widgets::navigatable::Navigatable;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Margin;
use ratatui::layout::Rect;
use ratatui::style;
use ratatui::symbols;
use ratatui::widgets;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

// picture size 225 x 320 ish

const PICTURE_RATIO: f32 = 225.0 / 320.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    NavBar,
    Content,
}

#[derive(Clone)]
pub struct ProfileScreen {
    user: User,
    navbar: NavBar,
    focus: Focus,
    image_manager: Arc<Mutex<ImageManager>>,
    bg_loaded: bool,
    app_info: ExtraInfo,
    navigation_fav: Navigatable,
}
impl ProfileScreen {
    pub fn new(info: ExtraInfo) -> Self {
        Self {
            focus: Focus::Content,
            navbar: NavBar::new()
                .add_screen(OVERVIEW)
                .add_screen(SEASONS)
                .add_screen(SEARCH)
                .add_screen(LIST)
                .add_screen(PROFILE),
            image_manager: Arc::new(Mutex::new(ImageManager::new())),
            bg_loaded: false,
            user: User::empty(),
            app_info: info,
            navigation_fav: Navigatable::new((2, 5)),
        }
    }
}

impl Screen for ProfileScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(widgets::Clear, area);

        // just the navbar bro
        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .areas(area);

        self.navbar.render(frame, top);

        // the actual screen
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
            .areas(bottom);

        //pfp
        let [pfp, info] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Fill(1),
            ])
            .areas(left);

        ImageManager::render_image(
            &self.image_manager,
            &self.user,
            frame,
            pfp.inner(Margin::new(1, 1)),
            false,
        );

        // favorite animes
        let [right_top, fav_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Percentage(40)])
            .areas(right);
        let block = Block::default()
            .border_set(symbols::border::ROUNDED)
            .borders(Borders::ALL)
            .title("Favorited Animes")
            .border_style(style::Style::default().fg(PRIMARY_COLOR));
        frame.render_widget(block, fav_area);

        self.navigation_fav.construct(
            &self.user.favorited_animes,
            fav_area.inner(Margin::new(1, 1)),
            |anime, area, selected| {

                if selected {
                    frame.render_widget(
                        Block::default()
                            .border_set(symbols::border::ROUNDED)
                            .borders(Borders::ALL)
                            .border_style(style::Style::default().fg(HIGHLIGHT_COLOR)),
                        area,
                    );
                }

                let [title, image] = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Fill(1),
                    ])
                    .areas(area.inner(Margin::new(1, 1)));

                // favorite anime title
                let title_text = Paragraph::new(anime.title.clone()).alignment(Alignment::Center);
                frame.render_widget(title_text, title);

                // favorite anime image

                let image_width = (image.height as f32 * PICTURE_RATIO * TERMINAL_RATIO) as u16;
                let centered_image_area = Rect::new(
                    image.x + (image.width - image_width) / 2, 
                    image.y,
                    image_width,
                    image.height,
                );

                ImageManager::render_image(
                    &self.image_manager,
                    anime, 
                    frame, 
                    centered_image_area, 
                    selected
                );
            },
        );


        // user information right side
        let block = Block::default()
            .border_set(symbols::border::ROUNDED)
            .borders(Borders::ALL)
            .title("User Statistics")
            .border_style(style::Style::default().fg(PRIMARY_COLOR));
        frame.render_widget(block, right_top);


        // user information left side

        let user_info_block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .title("User Info")
            .border_style(style::Style::default().fg(PRIMARY_COLOR));
        frame.render_widget(user_info_block, info);

        let user_info_text = vec![
            format!("Username: {}", self.user.name),
            format!("Joined: {}", self.user.joined_at),
            format!("Anime Count: {}", self.user.anime_statistics.num_items),
            format!("Episodes Count: {}", self.user.anime_statistics.num_episodes),
            format!("Days: {}", self.user.anime_statistics.num_days),
            format!("Score: {}", self.user.anime_statistics.mean_score),
        ];
        for (i, line) in user_info_text.iter().enumerate() {
            let paragraph = Paragraph::new(line.clone())
                .alignment(Alignment::Left)
                .block(Block::default().borders(Borders::NONE));
            frame.render_widget(paragraph, Rect::new(info.x + 1, info.y + 1 + i  as u16, info.width, 1));
        }
    }

    // handle inut function
    // for this spcific screen bro
    fn handle_input(&mut self, key_event: KeyEvent) -> Option<Action> {
        match self.focus {
            Focus::NavBar => {
                if let Some(action) = self.navbar.handle_input(key_event) {
                    return Some(action);
                }
            }
            Focus::Content => match key_event.code {
                _ => {
                    self.navbar.select();
                    self.focus = Focus::NavBar;
                }
            },
        }

        None
    }

    fn clone_box(&self) -> Box<dyn Screen + Send + Sync> {
        Box::new(self.clone())
    }

    fn background(&mut self) -> Option<JoinHandle<()>> {
        if self.bg_loaded {
            return None;
        }
        self.bg_loaded = true;

        let info = self.app_info.clone();
        let image_manager = self.image_manager.clone();
        let id = self.get_name();
        ImageManager::init_with_threads(&self.image_manager, info.app_sx.clone());

        Some(std::thread::spawn(move || {
            if let Some(user) = info.mal_client.get_user() {
                let username = user.name.clone();
                ImageManager::query_image_for_fetching(&image_manager, &user);
                let update = BackgroundUpdate::new(id.clone()).set("user", user);
                info.app_sx.send(Event::BackgroundNotice(update)).ok();

                if let Some(favorited_animes) = info.mal_client.get_favorited_anime(username) {
                    for anime in favorited_animes.clone() {
                        ImageManager::query_image_for_fetching(&image_manager, &anime);
                    }
                    let update =
                        BackgroundUpdate::new(id).set("favorited_animes", favorited_animes);
                    info.app_sx.send(Event::BackgroundNotice(update)).ok();
                }

                return;
            }
        }))
    }

    fn apply_update(&mut self, mut update: BackgroundUpdate) {
        if let Some(user) = update.take::<User>("user") {
            self.user = user;
        }
        if let Some(favorited_animes) = update.take::<Vec<FavoriteAnime>>("favorited_animes") {
            self.user.add_favorite_animes(favorited_animes);
        }
    }
}
