mod input_handler;
use std::sync::mpsc;
use crate::app::Event;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;


pub fn get_handlers() -> Vec<fn(mpsc::Sender<Event>)> {
    vec![
        input_handler::input_handler,
        // add more handlers here
    ]
}

