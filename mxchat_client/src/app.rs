use crate::{auth_page::AuthentificationPage, chat_page::ChatPage};

pub struct ChatApp {
    auth_page: AuthentificationPage,
    chat_page: Option<ChatPage>,
}

impl ChatApp {
    pub fn new() -> Self {
        Self {
            auth_page: AuthentificationPage::new(),
            chat_page: None
        }
    }
}

impl eframe::App for ChatApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        if let Some(chat_page) = &mut self.chat_page {
            chat_page.show(ctx);
        }
        else {
            self.chat_page = self.auth_page.show(ctx)
            .map(|(socket, user)| ChatPage::new(socket, user));
        }

    }
}