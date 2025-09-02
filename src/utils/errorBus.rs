use std::sync::mpsc::Sender;
use once_cell::sync::OnceCell;
use crate::app::Event;

static DISPATCH_TX: OnceCell<Sender<Event>> = OnceCell::new();

pub fn init(tx: Sender<Event>) {
    let _ = DISPATCH_TX.set(tx);
}

pub fn dispatch(ev: Event) {
    if let Some(tx) = DISPATCH_TX.get() {
        let _ = tx.send(ev);
    } else {
        eprintln!("[no bus] {:?}", ev);
    }
}

pub fn error<S: Into<String>>(msg: S) {
    dispatch(Event::ShowError(msg.into()));
}
