// File: src/gui/subscription.rs
use crate::gui::message::Message;
use crate::gui::state::{AppState, GuiApp};
use iced::{Subscription, event, keyboard, window};

pub fn subscription(app: &GuiApp) -> Subscription<Message> {
    use iced::keyboard::key;

    let mut subs = Vec::new();

    if matches!(app.state, AppState::Onboarding | AppState::Settings) {
        subs.push(keyboard::listen().filter_map(|event| {
            if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                if key == key::Key::Named(key::Named::Tab) {
                    return Some(Message::TabPressed(modifiers.shift()));
                }
            }
            None
        }));
    }

    // Track window metrics (Size)
    subs.push(event::listen_with(|evt, _status, _window_id| match evt {
        iced::Event::Window(window::Event::Resized(size)) => Some(Message::WindowResized(size)),
        _ => None,
    }));

    Subscription::batch(subs)
}
