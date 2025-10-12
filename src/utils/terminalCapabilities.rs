use ratatui_image::picker::Picker;
use std::{sync::OnceLock, thread, time::Duration};
use crate::send_error;

static GLOBAL_PICKER: OnceLock<Picker> = OnceLock::new();
pub const TERMINAL_RATIO: f32 = 2.20; // terminal "pixel" ratio (length to width of a single pixel)

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

    fn init_picker() -> Picker {
        let max_retries = 30;
        let retry_delay = Duration::from_millis(100);

        for attempt in 1..=max_retries {
            match Picker::from_query_stdio() {
                Ok(picker) => {
                    return picker;
                }
                Err(e) => {
                    send_error!(
                        "Attempt {}/{}: failed to initialize picker: {}",
                        attempt,
                        max_retries,
                        e
                    );
                    if attempt < max_retries {
                        thread::sleep(retry_delay);
                    }
                }
            }
        }
        panic!(
            "Failed to initialize Picker after {} attempts (quitting)",
            max_retries
        );
    }

    fn get_picker() -> &'static Picker {
        GLOBAL_PICKER.get_or_init(Self::init_picker)
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
