use eframe::egui;
use mxchat_core::messaging::Contact;

use super::{JobStatus, ShowMainContentSignal};

pub enum ContactPanelEvent {
    SendRequestContact(String),
    DisconnectUser
}

pub struct ContactsPanel {
    content_show_signal: Option<ShowMainContentSignal>,
    contacts: Vec<Contact>,
    searched_contact: Option<String>,
    contact_search_job_status: JobStatus,
    event: Option<ContactPanelEvent>,
    selected_contact: Option<usize>,
    username: String
}

impl ContactsPanel {
    pub fn new(username: &str) -> Self {
        Self {
            content_show_signal: None,
            contacts: Vec::new(),
            searched_contact: None,
            contact_search_job_status: JobStatus::Idle,
            event: None,
            selected_contact: None,
            username: username.into()
        }
    }

    pub fn main_content_signal(&self) -> Option<ShowMainContentSignal> {
        self.content_show_signal
    }

    pub fn next_event(&mut self) -> Option<ContactPanelEvent> {
        self.event.take()
    }

    pub fn add_contact(&mut self, contact: Contact) {
        self.contacts.push(contact);
        self.contact_search_job_status = JobStatus::Idle;
        if let Some(buffer) = self.searched_contact.as_mut() {
            buffer.clear();
        }
    }

    pub fn contact_search_failed(&mut self, error_message: &str) {
        println!("{error_message}");
        self.contact_search_job_status = JobStatus::Failed(error_message.to_string());
    }

    pub fn seletected_contact(&self) -> Option<&Contact> {
        self.selected_contact
            .and_then(|index| self.contacts.get(index))
    }

    pub fn show(&mut self, ctx: &egui::Context) {

        let width = ctx.screen_rect().width();

        egui::SidePanel::left("control_panel")
        .exact_width(width / 3.0)
        .show(ctx, |ui| {
            ui.label(format!("Connected as {}", self.username));
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

        if ui.button("Logout").clicked() {
            self.event = Some(ContactPanelEvent::DisconnectUser);

            return true;
        }

        false
    }

    fn show_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {

            let text_empty = self.searched_contact.as_ref()
                .unwrap().is_empty();

            let enable_add = !text_empty && self.contact_search_job_status != JobStatus::InProgress;
            if ui.add_enabled(enable_add, egui::Button::new("Add")).clicked() {
                let username = self.searched_contact.clone().unwrap();
                self.contact_search_job_status = JobStatus::InProgress;
                self.event = Some(ContactPanelEvent::SendRequestContact(username));
            }
        
            if ui.add_enabled(self.contact_search_job_status != JobStatus::InProgress, egui::Button::new("X")).clicked() {
                self.searched_contact = None;
                self.contact_search_job_status = JobStatus::Idle;
                return;
            }

            let text_edit = 
                    egui::TextEdit::singleline(self.searched_contact.as_mut().unwrap());

            ui.add_sized(ui.available_size(), text_edit);
        });
    }

    fn show_contacts(&mut self, ui: &mut egui::Ui) {
        let new_selected_contact= self.contacts
            .iter()
            .enumerate()
            .filter_map(|e| Self::show_contact(e, ui))
            .reduce(|acc, _| acc);

        if let Some(selected_contact) = new_selected_contact {
            self.selected_contact = Some(selected_contact);
            self.content_show_signal = Some(ShowMainContentSignal::Conversation);
        }
    }

    fn show_contact((contact_index, contact): (usize, &Contact), ui: &mut egui::Ui) -> Option<usize> {
        let label = egui::Label::new(&contact.nickname)
            .sense(egui::Sense::hover().union(egui::Sense::click()));

        let response = ui.add(label)
            .on_hover_cursor(egui::CursorIcon::PointingHand);


        if response.clicked() {
            Some(contact_index)
        }
        else {
            None
        }
    }
}