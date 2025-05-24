use std::sync::OnceLock;
use ratatui_image::picker::Picker;

static GLOBAL_PICKER: OnceLock<Picker> = OnceLock::new();

pub struct TerminalCapabilities {
    picker: &'static Picker,
}

#[allow(dead_code)]
impl TerminalCapabilities {
    pub fn instance() -> Self {
        Self {
            picker: Self::get_picker(),
        }
    }
 
    fn get_picker() -> &'static Picker {
        GLOBAL_PICKER.get_or_init(|| {
            Picker::from_query_stdio().expect("Failed to initialize Picker")
        })
    }
 
    pub fn picker(&self) -> &'static Picker {
        self.picker
    }
 
    // add methods to query terminal capabilities
    pub fn supports_images(&self) -> bool {
        // check if picker supports image protocols
        // implementation depends on ratatui_image API
        true // placeholder
    }
 
    pub fn max_colors(&self) -> u32 {
        // query color support
        256 // placeholder
    }
}

pub fn get_picker() -> &'static Picker {
    TerminalCapabilities::instance().picker()
}
