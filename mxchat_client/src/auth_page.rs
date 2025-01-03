use std::net::TcpStream;

use eframe::egui::{self, CursorIcon};
use mxchat_core::{auth::{User, UserConnectData, UserRegisterData}, notification::Notification};

use crate::{gui_utils::number_text_edit, networking::{read_notification, read_notification_payload, send_connect_cmd, send_register_cmd}};

pub struct AuthentificationPage {
    registration_page: RegistrationPage,
    login_page: LoginPage,
    registering: bool
}

impl AuthentificationPage {
    pub fn new() -> Self {
        Self {
            registration_page: RegistrationPage::new(),
            login_page: LoginPage::new(),
            registering: false
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<(TcpStream, User)> {

        egui::CentralPanel::default()
        .show(ctx, |ui| {
            ui.heading("Chat App");
            ui.separator();

            let result = if self.registering {
                self.registration_page.show(ui)
            }
            else {
                self.login_page.show(ui)
            };

            ui.add_space(30.0);
            self.switch_page_widgets(ui);

            result

        }).inner
    }

    fn switch_page_widgets(&mut self, ui: &mut egui::Ui) {
        let page = if self.registering {
            "login"
        }
        else {
            "registering"
        };

        let response = ui
            .label(format!("Go to {page} page"))
            .on_hover_cursor(CursorIcon::PointingHand);

        if response.clicked() {
            self.registering = !self.registering;
        }
    }
}

struct RegistrationPage {
    host_name: String,
    port: String,
    registration_data: UserRegisterData,
    error_message: Option<String>
}

impl RegistrationPage {
    fn new() -> Self {

        let registration_data = UserRegisterData {
            username: String::new(),
            nickname: String::new(),
            password: String::new(),
        };
        
        Self {
            host_name: String::from("127.0.0.1"),
            port: String::from("8080"),
            registration_data,
            error_message: None,
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) -> Option<(TcpStream, User)> {
        let result = egui::Grid::new("registration_form_grid")
        .max_col_width(500.0)
        .spacing((10.0, 20.0))
        .show(ui, |ui| {

            ui.label("Hostname");
            ui.text_edit_singleline(&mut self.host_name);

            ui.label("Port");
            number_text_edit(ui, 5, &mut self.port);

            ui.end_row();

            ui.label("Username");
            ui.text_edit_singleline(&mut self.registration_data.username);
            ui.end_row();

            ui.label("Nickname");
            ui.text_edit_singleline(&mut self.registration_data.nickname);
            ui.end_row();

            ui.label("Password");
            ui.text_edit_singleline(&mut self.registration_data.password);
            ui.end_row();



            let response = ui.add_enabled(
                self.is_form_valid(),
                egui::Button::new("Register")
            );

            if response.clicked() {
                self.register_and_connect()
            }
            else {
                None
            }
        }).inner;

        if result.is_some() {
            self.clear();
            return result;
        }

        if let Some(error_message) = &self.error_message {
            ui.separator();
            ui.label(error_message);
        }

        None

    }

    fn is_form_valid(&self) -> bool {
        !self.host_name.is_empty() &&
        !self.port.is_empty() &&
        !self.registration_data.username.is_empty() &&
        !self.registration_data.nickname.is_empty() &&
        !self.registration_data.password.is_empty()
    }

    fn create_tcp_connection(&self) -> Result<TcpStream, String> {
        let address = format!("{}:{}", self.host_name, self.port);
        let socket = TcpStream::connect(address)
            .map_err(|_| String::from("Could not connect to server"))?;

        Ok(socket)
    }

    fn register_user(&self, socket: &mut TcpStream) -> Result<(), String> {

        send_register_cmd(socket, self.registration_data.clone())
            .map_err(|_| String::from("Could not register user"))?;

        let notification = read_notification(socket)
                .map_err(|_| String::from("Error while connecting to server"))?;

        match notification {
            Notification::UserRegistred => Ok(()),
            Notification::UserAlreadyExist => Err(String::from("User already registered")),
            _ => Err(String::from("Error while connecting to server"))
        }
    }

    fn register_and_connect(&mut self) -> Option<(TcpStream, User)> {

        self.create_tcp_connection()
            .and_then(|mut socket| {
                self.register_user(&mut socket)?;

                let connect_data = UserConnectData {
                    username: self.registration_data.username.clone(),
                    password: self.registration_data.password.clone(),
                };

                let user = connect_user(&mut socket, connect_data)?;
                Ok((socket, user))
            })
            .map_err(|error_message| self.error_message = Some(error_message))
            .ok()
    }

    fn clear(&mut self) {
        self.registration_data.username.clear();
        self.registration_data.nickname.clear();
        self.registration_data.password.clear();
    }

}

struct LoginPage {
    host_name: String,
    port: String,
    connect_data: UserConnectData,

    error_message: Option<String>
}

impl LoginPage {
    fn new() -> Self {

        let connect_data = UserConnectData {
            password: String::new(),
            username: String::new()
        };

        Self {
            host_name: String::from("127.0.0.1"),
            port: String::from("8080"),
            connect_data,

            error_message: None
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) -> Option<(TcpStream, User)> {
        let result = egui::Grid::new("registration_form_grid")
        .max_col_width(500.0)
        .spacing((10.0, 20.0))
        .show(ui, |ui| {
            ui.label("Hostname");
            ui.text_edit_singleline(&mut self.host_name);

            ui.label("Port");
            number_text_edit(ui, 5, &mut self.port);

            ui.end_row();

            ui.label("Username");
            ui.text_edit_singleline(&mut self.connect_data.username);
            ui.end_row();

            ui.label("Password");
            ui.text_edit_singleline(&mut self.connect_data.password);
            ui.end_row();

            let response = ui
            .add_enabled(
                self.is_form_valid(), 
                egui::Button::new("Login")
            );

            if response.clicked() {
                self.connect_user()
            }
            else {
                None
            }
        }).inner;

        if result.is_some() {
            self.clear();
            return result;
        }

        if let Some(error_message) = &self.error_message {
            ui.separator();
            ui.label(error_message);
        }

        None
    }

    fn is_form_valid(&self) -> bool {
        !self.host_name.is_empty() &&
        !self.port.is_empty() &&
        !self.connect_data.username.is_empty() &&
        !self.connect_data.password.is_empty()
    }

    fn create_tcp_connection(&self) -> Result<TcpStream, String> {
        let address = format!("{}:{}", self.host_name, self.port);
        let socket = TcpStream::connect(address)
            .map_err(|_| String::from("Could not connect to server"))?;

        Ok(socket)
    }

    fn connect_user(&mut self) -> Option<(TcpStream, User)> {

        self.create_tcp_connection()
            .and_then(|mut socket| {
                let user = connect_user(&mut socket, self.connect_data.clone())?;
                Ok((socket, user))
            })
            .map_err(|error_message| self.error_message = Some(error_message))
            .ok()
    }

    fn clear(&mut self) {
        self.connect_data.username.clear();
        self.connect_data.password.clear();
    }
}

fn connect_user(socket: &mut TcpStream, connect_data: UserConnectData) -> Result<User, String> {

    send_connect_cmd(socket, connect_data)
            .map_err(|_| String::from("Could not connect user"))?;

    let notification = read_notification(socket)
            .map_err(|_| String::from("Error while connecting to server"))?;

    match notification {
        Notification::UserConnected => 
                read_notification_payload(socket)
                .ok()
                .and_then(|mut bytes_buffer| User::from_bytes(&mut bytes_buffer))
                .ok_or(String::from("Cannot read user data from server"))
                .inspect(|user| println!("User = {user:?}")),
        Notification::UserIsAlreadyConnected => 
            Err(String::from("User is already connected")),
        Notification::UserNotFound =>
            Err(String::from("User is not registered")),
        Notification::UserPasswordIncorrect =>
            Err(String::from("Password is incorrect")),
        _ => Err(String::from("Error while connecting to server"))
    }
}