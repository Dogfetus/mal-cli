#![allow(non_snake_case)]

use std::path::PathBuf;
pub mod terminalCapabilities;
pub mod stringManipulation;
pub mod imageManager;
pub mod customThreadProtocol;
pub mod input;
pub mod functionStreaming;
pub mod store;
pub mod errorBus;

pub fn get_app_dir() -> PathBuf {
    std::env::var("HOME").ok()
    .map(|home| PathBuf::from(home)
    .join(".local/share/mal-cli"))
    .expect("Failed to get app directory")
} 



#[macro_export]
macro_rules! send_error {
    ($msg:literal $(,)? ) => {{
        $crate::utils::errorBus::error($msg);
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        $crate::utils::errorBus::error(::std::format!($fmt, $($arg)*));
    }};
}
