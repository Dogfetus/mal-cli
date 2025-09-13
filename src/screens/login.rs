use std::cmp::{max, min};
use crate::{add_screen_caching, app::Event, mal::MalClient, screens::widgets::button::Button};
use crossterm::event::{KeyCode, KeyEvent};
use super::{screens::*, BackgroundUpdate, ExtraInfo, Screen};
use std::thread::JoinHandle;
use crate::app::Action;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect}, 
    widgets::{ Block, Borders, Clear, Paragraph}, 
    style::{Color, Modifier, Style}, 
    Frame 
};


//TODO: option to copy the url to clipboard
#[derive(Clone)]
pub struct LoginScreen { 
    selected_button: usize,
    buttons: Vec<&'static str>,
    login_url: String,
    app_info: ExtraInfo 
}

impl LoginScreen {
    pub fn new(info: ExtraInfo) -> Self {
        Self {
            selected_button: 0,
            buttons: vec![
                "nothing yet",
                "Back",
            ],
            login_url: String::new(),
            app_info: info
        }
    }
}

impl Screen for LoginScreen {
    add_screen_caching!();

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        frame.render_widget(Clear, area);

        let page_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);


        let header_text = vec![
            r"                                  #     %%%                                     ",
            r"                                @&@#   #%% #&                                   ",
            r"                   @&       &&#&%@@%  %@&&#@@#                                  ",
            r"                 @@##&&&@@ &&@;&%#   #|@#&#&@% ##                               ",
            r"              #%&&@&%%# &@   %|#&@%|~  %@###@&&%                                ",
            r"                &&@%@%@#&%  / @&%@%#   ##%%#% @&@                               ",
            r"         &@ #@#|&@%~%###&&##@#&@   #  __:_&%@  %##                              ",
            r"           @&@%#@ @ %%|&_@#%#      #:_     |%#  &@                              ",
            r"          @@#@@|@  &~/    @%&~    /     &&@@__#@%&#                             ",
            r"         %#%%@%#\&   :   #  % \\\/         |@&%&&                               ",
            r"          @ ##@#%&__ |      @   |          % @&  @                              ",
            r"        %@@  &&&@@# _~_=         |           #&                                 ",
            r"          &&%%& %# @    \=        =                                             ",
            r"        ##@~\;& &#       /:_::;_____                                            ",
            r"      %&## @#   \__=  //=          \;                                     @     ",
            r"       #    &       =_               \\          ________=             @&&%@@%# ",
            r"                                       \\    _~_:____     ___~          #@;%@%@ ",
            r"                                         =~_~;_~:              \       # |##%&# ",
            r"                                         =_\=                    ___: /:~_;#%&&@",
            r"                                         ~||                     |   =|   &&@%@@",
            r"                                         |;|                      :    |  @   &%",
            r"                                        |;|              &#@%    | \|%;%%       ",
            r"                                        |~|                %@  _| @&##% @@%%    ",
            r"                                        ;:=                  &%& @&%%&@%&@&     ",
            r"                                       |||                  &&%%#@&#&%%@# @     ",
            r"                           \__.-.____./~=|\..________/         @#&@#@@&&##      ",
            r"                            \         *    *..      /           &  @&%@&        ",
            r"                             \_____________________/                            ",
            r"                               ‾                 ‾                              ",
        ];

        let alpha = Paragraph::new(header_text.join("\n"))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

        frame.render_widget(alpha, page_chunk[0]);

        let text_field_area = Rect::new(
            page_chunk[1].x + min(page_chunk[1].width / 2 - 25, page_chunk[1].width / 4),
            page_chunk[1].y + 2,
            max(page_chunk[1].width / 2, 50),
            3);

        // if self.login_url.is_empty() {
        //     self.login_url = init_oauth().0;
        // }

        let url_field = Paragraph::new(self.login_url.clone())
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);

        frame.render_widget(url_field, text_field_area);

        for (i, button) in self.buttons.iter().enumerate() {
            Button::new(button)
                .offset((0, -3 + (i as i16 * 3)))
                .center()
                .selected(i == self.selected_button)
                .render(frame, page_chunk[1]);
        }

    }

    fn handle_keyboard(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('j') => {
                if self.selected_button > 0 {
                    self.selected_button -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('k') => {
                if self.selected_button < self.buttons.len() - 1 {
                    self.selected_button += 1;
                }
            }
            KeyCode::Enter => {
                match self.selected_button {
                    _ => { 
                        if MalClient::user_is_logged_in() {
                            self.login_url.clear();
                        }
                        return Some(Action::SwitchScreen(LAUNCH));
                    }
                }
            }
            _ => {} 
        };
        None
    }

    fn background(&mut self) -> Option<JoinHandle<()>> {
        if MalClient::user_is_logged_in() {
            return None;
        }

        let login_url = self.login_url.clone();
        let id = self.get_name();
        let info = self.app_info.clone();
        let mal_client = info.mal_client.clone(); 

        Some(std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            {

                if !login_url.is_empty() {
                    return;
                }
            }

            let (url_to_print, joinable) = MalClient::init_oauth();

            for i in 0..url_to_print.len()+1 {
                std::thread::sleep(std::time::Duration::from_millis(8));
                let new_url = url_to_print[0..i].to_string();
                let update = BackgroundUpdate::new(id.clone())
                    .set("login_url", new_url);
                let _ = info.app_sx.send(Event::BackgroundNotice(update));
            }

            joinable.join().unwrap();  
            mal_client.update_user_login();
            let new_url = "Login successful".to_string();
            let update = BackgroundUpdate::new(id.clone())
                .set("login_url", new_url);
            let _ = info.app_sx.send(Event::BackgroundNotice(update));
        }))
    }

    fn apply_update(&mut self, update: BackgroundUpdate) {
        if let Some(url) = update.get::<String>("login_url") {
            self.login_url = url.clone();
        }
    }

    fn uses_navbar(&self) -> bool {
        false
    }
}
