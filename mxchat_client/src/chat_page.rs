use std::{net::TcpStream, sync::Arc, thread};

use eframe::egui;
use mxchat_core::{auth::User, io::BytesBuffer, messaging::Contact};

use crate::{networking::{read_notification, read_notification_payload, send_request_contact_cmd}, notifications_handler::{ChatNotificationHandler, NotificationHandlerSignal, NotificationsQueue}};

#[derive(Eq, PartialEq)]
enum JobStatus {
    Idle,
    InProgress,
    Failed(String),
}

#[derive(Clone, Copy)]
enum ShowMainContentSignal {
    UserData,
}

struct ContactsPanel {
    content_show_signal: Option<ShowMainContentSignal>,
    contacts: Vec<Contact>,
    selected_contact: Option<usize>,
    searched_contact: Option<String>,
    contact_search_job_status: JobStatus,
    socket: TcpStream,
}

impl ContactsPanel {
    fn new(socket: TcpStream) -> Self {
        Self {
            content_show_signal: None,
            contacts: Vec::new(),
            selected_contact: None,
            searched_contact: None,
            socket,
            contact_search_job_status: JobStatus::Idle,
        }
    }

    fn show(&mut self, ctx: &egui::Context) -> Option<ShowMainContentSignal> {

        let width = ctx.screen_rect().width();

        egui::SidePanel::left("control_panel")
        .exact_width(width / 3.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.searched_contact.is_none() {
                    self.show_controls(ui);
                }
                else {
                    self.show_search_bar(ui);
                }
            });

            if let JobStatus::Failed(error_message) = &self.contact_search_job_status {
                ui.label(error_message);
            }
            
            ui.separator();
            self.show_contacts(ui);
        });

        self.content_show_signal
    }

    fn show_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Chat App");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {

            ui.menu_button(":", |ui| {
                if self.show_menu_options(ui) {
                    ui.close_menu();
                }
            });

            if ui.button("+").clicked() {
                self.searched_contact = Some(String::new())
            }
        });
    }

    fn show_menu_options(&mut self, ui: &mut egui::Ui) -> bool {
        if ui.button("View Info").clicked() {
            self.content_show_signal = Some(ShowMainContentSignal::UserData);
            return true;
        }
        false
    }

    fn show_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {

            let text_empty = self.searched_contact.as_ref()
                .unwrap().is_empty();

            let enable_crtls = !text_empty && self.contact_search_job_status != JobStatus::InProgress;

            if ui.add_enabled(enable_crtls, egui::Button::new("Add")).clicked() {
                let username = self.searched_contact.clone().unwrap();
                let result = send_request_contact_cmd(&mut self.socket, username);
                if result.is_err() {
                    self.contact_search_job_status = JobStatus::Failed("Error while connecting to server".into());
                }
                else {
                    self.contact_search_job_status = JobStatus::InProgress;
                }
            }
        
            if ui.add_enabled(enable_crtls, egui::Button::new("X")).clicked() {
                self.searched_contact = None;
                self.contact_search_job_status = JobStatus::Idle;
                return;
            }

            let text_edit = 
                    egui::TextEdit::singleline(self.searched_contact.as_mut().unwrap());

            ui.add_sized(ui.available_size(), text_edit);
        });
    }

    fn show_contacts(&self, ui: &mut egui::Ui) {
        self.contacts
            .iter()
            .for_each(|contact| Self::show_contact(contact, ui));
    }

    fn show_contact(contact: &Contact, ui: &mut egui::Ui) {
        ui.label(&contact.nickname);
    }

    fn add_contact(&mut self, contact: Contact) {
        self.contacts.push(contact);
        self.contact_search_job_status = JobStatus::Idle;
        if let Some(buffer) = self.searched_contact.as_mut() {
            buffer.clear();
        }
    }


    fn contact_search_failed(&mut self, error_message: &str) {
        println!("{error_message}");
        self.contact_search_job_status = JobStatus::Failed(error_message.to_string());
    }
}



pub struct ChatPage {
    socket: TcpStream,
    current_user: User,
    contacts_panel: ContactsPanel,

    notifications_queue: Arc<NotificationsQueue>,
}

impl ChatPage {
    pub fn new(socket: TcpStream, current_user: User) -> Self {
        let socket_clone = socket.try_clone().unwrap();
        let notifications_queue = Arc::new(NotificationsQueue::new());

        run_notification_listener_par(
            Arc::clone(&notifications_queue),
            socket.try_clone().unwrap()
        );

        Self {
            socket,
            current_user,
            contacts_panel: ContactsPanel::new(socket_clone),
            notifications_queue
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.handle_notifications();
        if let Some(content_show_signal) = self.contacts_panel.show(ctx) {
            self.show_central_panel(ctx, content_show_signal);
        }
    }

    fn show_central_panel(&mut self, ctx: &egui::Context, content_show_signal: ShowMainContentSignal) {
        egui::CentralPanel::default()
        .show(ctx, |ui| {
            match content_show_signal {
                ShowMainContentSignal::UserData => self.show_user_data(ui),
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