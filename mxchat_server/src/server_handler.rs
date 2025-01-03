use std::{collections::HashMap, net::TcpStream, sync::RwLock};

use mxchat_core::{auth::{User, UserConnectData, UserId}, io::BytesBuffer, messaging::Contact, notification::Notification, utils::u32_as_bytes};

use crate::{command_handler::CommandHandler, server::{ServerConnectionData, ServerResponse}, user::{InMemoryUserRepository, UserData, UserIdGenerator, UserRepository}};

pub struct ServerCommandHandler {
    users_repo: Box<RwLock<dyn UserRepository>>,
    ids_generator: UserIdGenerator,
    users_sockets: RwLock<HashMap<UserId, TcpStream>>
}

impl ServerCommandHandler {
    pub fn new() -> Self {
        Self {
            users_repo: Box::new(RwLock::new(InMemoryUserRepository::new())),
            ids_generator: UserIdGenerator::new(),
            users_sockets: RwLock::new(HashMap::new()),
        }
    }

    fn add_user(&self, user: UserData) {
        self
            .users_repo
            .write()
            .unwrap()
            .add_user(user);
    }

    fn register_socket(&self, user_id: UserId, socket: TcpStream) {
        self.users_sockets
            .write()
            .unwrap()
            .insert(user_id, socket);
    }
}

impl CommandHandler for ServerCommandHandler {
    fn handle_register_cmd(&self, user_register_data: mxchat_core::auth::UserRegisterData) -> ServerResponse {

        println!("Registering user");

        let user_registered = self.users_repo
            .read()
            .unwrap()
            .find_user_with_username(&user_register_data.username)
            .is_some();

        if user_registered {
            return Notification::UserAlreadyExist.into();
        }

        let user_data = UserData {
            user: User {
                id: self.ids_generator.next_id(),
                username: user_register_data.username.clone(),
                nickname: user_register_data.nickname.clone()
            },
            password: user_register_data.password.clone()
        };

        self.add_user(user_data);

        Notification::UserRegistred.into()
    }
    
    fn handle_connect_cmd(&self, user_connect_data: UserConnectData, connection_data: &mut ServerConnectionData) -> ServerResponse {

        if connection_data.user_id.is_some() {
            return Notification::UserIsAlreadyConnected.into()
        }

        let result = self.users_repo
            .read()
            .unwrap()
            .find_user_with_username(&user_connect_data.username)
            .ok_or(Notification::UserNotFound)
            .and_then(|user| {
                if user.password == user_connect_data.password {
                    Ok(user)
                }
                else {
                    Err(Notification::UserPasswordIncorrect)
                }
            })
            .map(|user|{
                connection_data.user_id = Some(user.user.id);
                self.register_socket(user.user.id, connection_data.socket.try_clone().unwrap());

                let mut bytes_buffer = BytesBuffer::empty();
                let user_bytes = user.user.to_bytes();
                bytes_buffer.write_bytes(&u32_as_bytes(user_bytes.len() as u32));
                bytes_buffer.write_bytes(&user_bytes);

                ServerResponse::new(Notification::UserConnected, bytes_buffer)
            });


        match result {
            Ok(response) => response,
            Err(notif) => notif.into()
        }
    }
    
    fn handle_request_contact_cmd(&self, username: &str) -> ServerResponse {
        self.users_repo
            .read()
            .unwrap()
            .find_user_with_username(username)
            .map(|user| &user.user)
            .map(|user| Contact {
                id: user.id,
                nickname: user.nickname.clone(),
            })
            .map(|contact| {
                let mut bytes_buffer = BytesBuffer::empty();
                let bytes = contact.to_bytes();
                bytes_buffer.write_bytes(&u32_as_bytes(bytes.len() as u32));
                bytes_buffer.write_bytes(&bytes);

                ServerResponse::new(Notification::ReceiveContactInfo, bytes_buffer)
            })
            .unwrap_or(Notification::UserNotFound.into())        
    }
}