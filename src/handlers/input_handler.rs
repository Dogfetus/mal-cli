use std::sync::mpsc;
use crate::app::Event;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;



pub fn input_handler(sx: mpsc::Sender<Event>, stop: Arc<AtomicBool>) {
    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
        if let Ok(event) = crossterm::event::read() {
            match event {
                crossterm::event::Event::Key(key_event) => {
                    sx.send(Event::KeyPress(key_event)).unwrap();

                    // handle quit  TODO: (change to something better)
                    if key_event.kind == crossterm::event::KeyEventKind::Press &&
                        key_event.code == crossterm::event::KeyCode::Char('c') &&
                        key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                    {
                        break;
                    }
                }

                crossterm::event::Event::Mouse(mouse_event) => {
                    match mouse_event.kind {
                        crossterm::event::MouseEventKind::Down(_) | 
                        crossterm::event::MouseEventKind::Up(_) |
                        crossterm::event::MouseEventKind::Drag(_) => {
                            sx.send(Event::MouseClick(mouse_event)).unwrap();
                            // sx.send(Event::MousePosition(x, y, mouse_event.kind)).unwrap();??
                        }
                        _ => {}
                    }
                }

                crossterm::event::Event::Resize(width, height) => {
                    sx.send(Event::Resize(width, height)).unwrap();
                }

                _ => {
                }
            }
        }
    }
}
