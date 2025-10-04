use std::sync::mpsc;
use crate::app::Event;


pub fn input_handler(sx: mpsc::Sender<Event>) {
    loop {
        if let Ok(event) = crossterm::event::read() {
            match event {
                crossterm::event::Event::Key(key_event) => {
                    if sx.send(Event::Input(event)).is_err() {
                        // this happens when the receiver is dropped
                        return;
                    }

                    // handle quit  TODO: (change to something better)
                    if key_event.kind == crossterm::event::KeyEventKind::Press &&
                        key_event.code == crossterm::event::KeyCode::Char('c') &&
                        key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                    {
                        return;
                    }
                }

                crossterm::event::Event::Mouse(mouse_event) => {
                    match mouse_event.kind {
                        // as long as we only need some of the keyinputs:
                        crossterm::event::MouseEventKind::ScrollUp |
                        crossterm::event::MouseEventKind::ScrollDown |
                        crossterm::event::MouseEventKind::ScrollRight |
                        crossterm::event::MouseEventKind::ScrollLeft |
                        crossterm::event::MouseEventKind::Moved |
                        crossterm::event::MouseEventKind::Down(_) => {
                            if sx.send(Event::Input(event)).is_err() {
                                return;
                            }
                        } 
                        _ => {}
                    }
                }

                crossterm::event::Event::Resize(width, height) => {
                    if sx.send(Event::Resize(width, height)).is_err() {
                        return;
                    }
                }

                _ => {
                }
            }
        }
    }
}
