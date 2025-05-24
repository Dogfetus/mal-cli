use ratatui::{
    layout::Rect, 
    Frame
};
use ratatui_image::{StatefulImage, protocol::StatefulProtocol};
use crate::utils::terminalCapabilities::get_picker;

pub struct CustomImage {
    image_state: StatefulProtocol,
}

impl CustomImage {
    pub fn new(image_path: &str) -> Self {

        let picker = get_picker(); 
        let dyn_img = image::ImageReader::open(image_path).expect("nah").decode().expect("dont want to");
        let image = picker.new_resize_protocol(dyn_img);

        Self {
            image_state: image,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(StatefulImage::default(), area, &mut self.image_state);
    }

}
