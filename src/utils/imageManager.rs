use crate::{
    app::Event,
    mal::{models::anime::Anime, network::fetch_image},
    screens::BackgroundUpdate,
    utils::terminalCapabilities::get_picker,
};
use ratatui::layout::Rect;
use ratatui_image::errors::Errors;
use ratatui_image::{
    StatefulImage,
    thread::{ResizeRequest, ResizeResponse, ThreadProtocol},
};
use std::collections::HashMap;
use std::sync::mpsc::{Sender, channel};

pub struct ImageManager {
    protocols: HashMap<usize, ThreadProtocol>,
    app_sx: Option<Sender<Event>>,
    id: Option<String>,
}

impl ImageManager {
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
            app_sx: None,
            id: None,
        }
    }

    pub fn init(&mut self, app_sx: Sender<Event>, id: String) {
        self.app_sx = Some(app_sx);
        self.id = Some(id);
    }

    // this will create a new trhead for each image
    pub fn prepare_image(&mut self, anime: &Anime) {
        if self.app_sx.is_none() || self.id.is_none() || self.protocols.contains_key(&anime.id) {
            return;
        }

        let anime = anime.clone();
        let app_sx = self.app_sx.as_ref().unwrap().clone();
        let id = self.id.clone().unwrap();

        std::thread::spawn(move || {
            let (image_tx, image_rx) = channel::<ResizeRequest>();

            match fetch_image(anime.main_picture.medium) {
                Ok(dyn_img) => {
                    let picker = get_picker();
                    let protocol = picker.new_resize_protocol(dyn_img);
                    let thread_protocol = ThreadProtocol::new(image_tx, Some(protocol));

                    let update = BackgroundUpdate::new(id)
                        .set("anime_id", anime.id)
                        .set("thread_protocol", thread_protocol);
                    let _ = app_sx.send(Event::BackgroundNotice(update));
                }
                Err(e) => {
                    eprintln!("Failed to fetch image: {}", e);
                    return;
                }
            }

            while let Ok(request) = image_rx.recv() {
                let result = request.resize_encode();
                let _ = app_sx.send(Event::ImageRedraw(anime.id, result));
            }
        });
    }

    pub fn load_image(&mut self, id: usize, protocol: ThreadProtocol) {
        if self.app_sx.is_none() {
            eprintln!("App sender is not initialized");
            return;
        }
        self.protocols.insert(id, protocol);
    }

    #[allow(dead_code)]
    pub fn load_image_from_file(&mut self, id: usize, image_path: &str) {
        if let Some(app_sx) = &self.app_sx {
            if let Ok(dyn_img) = image::open(image_path) {
                let picker = get_picker();
                let protocol = picker.new_resize_protocol(dyn_img);
                let (image_tx, image_rx) = channel::<ResizeRequest>();
                let app_sx = app_sx.clone();

                let thread_protocol = ThreadProtocol::new(image_tx, Some(protocol));
                self.protocols.insert(id, thread_protocol);

                std::thread::spawn(move || {
                    while let Ok(request) = image_rx.recv() {
                        let result = request.resize_encode();
                        let _ = app_sx.send(Event::ImageRedraw(id, result));
                    }
                });
            } else {
                eprintln!("Failed to open image at path: {}", image_path);
            }
        }
    }

    pub fn remove_image(&mut self, id: usize) {
        self.protocols.remove(&id);
    }

    pub fn render_image(&mut self, id: usize, frame: &mut ratatui::Frame, area: Rect) {
        if let Some(protocol) = self.protocols.get_mut(&id) {
            frame.render_stateful_widget(StatefulImage::new(), area, protocol);
        }
    }

    pub fn update_image(&mut self, id: usize, response: Result<ResizeResponse, Errors>) -> bool {
        if let Some(protocol) = self.protocols.get_mut(&id) {
            match response {
                Ok(completed) => protocol.update_resized_protocol(completed),
                Err(e) => {
                    eprintln!("failed to update image {}: error: {}", id, e);
                    false
                }
            }
        } else {
            eprintln!("Image with ID {} not found", id);
            false
        }
    }
}
