use ratatui_image::{StatefulImage, picker::Picker, thread::{ResizeRequest, ResizeResponse, ThreadProtocol}};
use std::sync::mpsc::{Sender, Receiver, channel};
use ratatui_image::errors::Errors;
use std::collections::HashMap;
use crate::{app::Event, utils::terminalCapabilities::get_picker};

pub struct ImageManager {
    protocols: HashMap<usize, ThreadProtocol>,
    app_sx: Option<Sender<Event>>,
}

impl ImageManager {
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
            app_sx: None,
        }
    }

    pub fn init(&mut self, app_sx: Sender<Event>) {
        self.app_sx = Some(app_sx);
    }

    // this will craete a new trhead for each image
    pub fn load_image(&mut self, id: usize, image_path: &str) {
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
            }
        }
    }

    pub fn remove_image(&mut self, id: usize) {
        self.protocols.remove(&id);
    }

    pub fn render_image(&mut self, frame: &mut ratatui::Frame, id: usize, area: ratatui::layout::Rect) {
        if let Some(protocol) = self.protocols.get_mut(&id) {
            frame.render_stateful_widget(StatefulImage::new(), area, protocol);
        }
    }

    pub fn update_image(&mut self, id: usize, response: Result<ResizeResponse, Errors>) -> bool {
        if let Some(protocol) = self.protocols.get_mut(&id) {
            match response {
                Ok(completed) => {
                    protocol.update_resized_protocol(completed)
                },
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
