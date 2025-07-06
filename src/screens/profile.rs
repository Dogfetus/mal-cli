use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;

use crate::app::Action;
use crate::app::Event;
use crate::mal::models::user::User;
use crate::utils::imageManager::ImageManager;

use super::BackgroundUpdate;
use super::Screen;
use super::screens::*;
use super::widgets::navbar::NavBar;

use crossterm::event::KeyEvent;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style;
use ratatui::widgets;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;

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
}
impl ProfileScreen {
    pub fn new() -> Self {
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
        }
    }
}

impl Screen for ProfileScreen {
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(widgets::Clear, area);

        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .areas(area);

        self.navbar.render(frame, top);

        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
            .areas(bottom);

        let test = Paragraph::new(self.user.name.clone())
            .style(style::Style::default().fg(style::Color::White))
            .block(Block::default().borders(Borders::ALL).title("User Info"));

        let block = Block::new()
            .borders(Borders::ALL)
            .style(style::Style::default().fg(style::Color::LightBlue));

        let [pfp, info, buttons] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
            ])
            .areas(left);

        frame.render_widget(test, left);
        frame.render_widget(block.clone(), right);
        frame.render_widget(block.clone(), info);
        frame.render_widget(block, buttons);
        ImageManager::render_image(&self.image_manager, &self.user, frame, pfp);
    }

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

    fn background(&mut self, info: super::BackgroundInfo) -> Option<JoinHandle<()>> {
        if self.bg_loaded {
            return None;
        }
        self.bg_loaded = true;

        let image_manager = self.image_manager.clone();
        let id = self.get_name();
        ImageManager::init_with_dedicated_thread(&self.image_manager, info.app_sx.clone(), id.clone());

        Some(std::thread::spawn(move || {
            if let Some(user) = info.mal_client.get_user() {
                ImageManager::fetch_image_sequential(&image_manager, &user);
                let update = BackgroundUpdate::new(id)
                    .set("user", user);
                info.app_sx.send(Event::BackgroundNotice(update)).unwrap();
                return;
            }
        }))
    }

    fn apply_update(&mut self, mut update: BackgroundUpdate) {
        if let Some(user) = update.take::<User>("user") {
            self.user = user;
        }
    }
}
