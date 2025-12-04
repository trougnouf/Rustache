// File: ./src/gui/subscription.rs
use crate::gui::message::Message;
use crate::gui::state::{AppState, GuiApp};
use iced::{Subscription, keyboard};

pub fn subscription(app: &GuiApp) -> Subscription<Message> {
    use iced::keyboard::key;

    if matches!(app.state, AppState::Onboarding | AppState::Settings) {
        return keyboard::on_key_press(|k, modifiers| {
            if k == key::Key::Named(key::Named::Tab) {
                Some(Message::TabPressed(modifiers.shift()))
            } else {
                None
            }
        });
    }
    Subscription::none()
}
