mod contacts_panel;

use std::{net::TcpStream, sync::Arc, thread};

use contacts_panel::{ContactPanelEvent, ContactsPanel};
use eframe::egui;
use mxchat_core::{auth::User, io::BytesBuffer};

use crate::{messenger::{MessagingInstance, Messenger}, networking::{read_notification, read_notification_payload, send_request_contact_cmd}, notifications_handler::{ChatNotificationHandler, NotificationHandlerSignal, NotificationsQueue}};

#[derive(Eq, PartialEq)]
pub enum JobStatus {
    Idle,
    InProgress,
    Failed(String),
}

#[derive(Clone, Copy)]
pub enum ShowMainContentSignal {
    UserData,
    Conversation,
}

pub struct ChatPage {
    socket: TcpStream,
    current_user: User,
    contacts_panel: ContactsPanel,
    exit: bool,

    messenger: Messenger,

    notifications_queue: Arc<NotificationsQueue>,
}

impl ChatPage {
    pub fn new(socket: TcpStream, current_user: User) -> Self {
        let notifications_queue = Arc::new(NotificationsQueue::new());

        run_notification_listener_par(
            Arc::clone(&notifications_queue),
            socket.try_clone().unwrap()
        );

        let contacts_panel = ContactsPanel::new(&current_user.username);

        Self {
            socket,
            current_user,
            contacts_panel,
            notifications_queue,
            exit: false,
            messenger: Messenger::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        self.handle_notifications();
        self.contacts_panel.show(ctx);
        if let Some(content_show_signal) = self.contacts_panel.main_content_signal() {
            self.show_central_panel(ctx, content_show_signal);
        }

        if let Some(event) = self.contacts_panel.next_event() {
            self.handle_contact_panel_event(event);
        }

        self.exit
    }

    fn show_central_panel(&mut self, ctx: &egui::Context, content_show_signal: ShowMainContentSignal) {
        egui::CentralPanel::default()
        .show(ctx, |ui| {
            match content_show_signal {
                ShowMainContentSignal::UserData => self.show_user_data(ui),
                ShowMainContentSignal::Conversation => self.show_conversation(ui)
            }
        });
    }

    fn show_user_data(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| ui.heading(&self.current_user.nickname));
        ui.separator();

        egui::Grid::new("user_infos")
        .show(ui, |ui| {
            ui.label("Username");
            ui.label(&self.current_user.username);
            ui.end_row();

            ui.label("Nickname");
            ui.label(&self.current_user.nickname);
        });
    }

    fn show_conversation(&mut self, ui: &mut egui::Ui) {
        let selected_contact = self.contacts_panel.seletected_contact().unwrap();
        let title = format!("Chat with {}", selected_contact.nickname);
        ui.vertical_centered(|ui| ui.heading(title));
        ui.separator();

        let instance = self.messenger.get_messaging_instance(selected_contact.id).unwrap();

        Self::show_chat_controls(ui, instance);
        
        egui::ScrollArea::vertical()
        .auto_shrink(false)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for _ in 0..100 {
                ui.label("Hello world!");
            }
        });
    }

    fn show_chat_controls(ui: &mut egui::Ui, instance: &mut MessagingInstance) {
        egui::TopBottomPanel::bottom("chat_crtls_panel")
        .min_height(75.0)
        .show_inside(ui, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            let _ = ui.button("send");
            let message_area = egui::TextEdit::multiline(&mut instance.text_to_send);
            egui::ScrollArea::vertical()
            .show(ui, |ui| ui.add_sized(ui.available_size(), message_area));
        });
        });
    }

    fn handle_notifications(&mut self) {
        let result = self.notifications_queue.pop_notification();
        if result.is_none() {
            return;
        }

        let (notification, payload) = result.unwrap();

        println!("Received notification {:?}", notification);

        let signal = ChatNotificationHandler::handle_notification(notification, payload);

        match signal {
            NotificationHandlerSignal::ContactReceived(contact) => {
                if contact.id != self.current_user.id {
                    self.messenger.add_messsaging_instance(contact.id);
                    self.contacts_panel.add_contact(contact);
                }
                else {
                    self.contacts_panel.contact_search_failed("You can't add yourself as contact!");
                }
            }
            NotificationHandlerSignal::ContactRetreivingFailed(error_message) => 
                self.contacts_panel.contact_search_failed(&error_message),
            
            _ => ()
        }
    }

    fn handle_contact_panel_event(&mut self, event: ContactPanelEvent) {
        match event {
            ContactPanelEvent::SendRequestContact(username) => {
                if send_request_contact_cmd(&mut self.socket, username).is_err() {
                    self.contacts_panel.contact_search_failed("Error while connecting to server");
                }
            }
            ContactPanelEvent::DisconnectUser => {
                self.exit = true;
            }
        }
    }
}


fn run_notification_listener(notifications_queue: Arc<NotificationsQueue>, mut socket: TcpStream) {
    loop {
        let result = read_notification(&mut socket)
        .and_then(|notification| {
            let payload = if notification.has_payload() {
                read_notification_payload(&mut socket)?
            }
            else {
                BytesBuffer::empty()
            };
            Ok((notification, payload))
        });

        if let Ok((notification, payload)) = result {
            notifications_queue.push_notification(notification, payload);
        }

    }
}

fn run_notification_listener_par(notifications_queue: Arc<NotificationsQueue>, socket: TcpStream) {
    thread::spawn(|| {
        run_notification_listener(notifications_queue, socket);
    });
}