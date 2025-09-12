#![allow(non_snake_case)]

pub mod terminalCapabilities;
pub mod stringManipulation;
pub mod imageManager;
pub mod customThreadProtocol;
pub mod input;
pub mod functionStreaming;
pub mod store;
pub mod errorBus;

#[macro_export]
macro_rules! send_error {
    ($msg:literal $(,)? ) => {{
        $crate::utils::errorBus::error($msg);
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        $crate::utils::errorBus::error(::std::format!($fmt, $($arg)*));
    }};
}
