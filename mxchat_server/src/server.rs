use std::{io::{self, Write}, net::{IpAddr, TcpListener, TcpStream, ToSocketAddrs}, sync::Arc, thread};

use mxchat_core::{auth::UserId, command::CommandParsingError, io::BytesBuffer, notification::Notification};

use crate::command_handler::{self, handle_command, CommandHandler, CommandHandlerRef};

pub struct ServerConnectionData {
    pub socket: TcpStream,
    pub user_id: Option<UserId>
}

pub enum ServerError {
    MissingCommandData,
    CommandParsingError(CommandParsingError),
    IoError(std::io::Error)
}

impl From<std::io::Error> for ServerError {
    fn from(value: std::io::Error) -> Self {
        ServerError::IoError(value)
    }
}

pub struct ServerConfig {
    pub address: IpAddr,
    pub port: u16
}

impl ServerConfig {
    pub fn as_socket_addr(&self) -> impl ToSocketAddrs {
        format!("{}:{}", self.address, self.port)
    }
}

pub struct ServerResponse {
    notification: Notification,
    data_bytes: BytesBuffer
}

impl ServerResponse {
    pub fn new(notification: Notification, data_bytes: BytesBuffer) -> Self {
        Self {
            notification,
            data_bytes
        }
    }
}

impl From<Notification> for ServerResponse {
    fn from(value: Notification) -> Self {
        Self::new(value, BytesBuffer::empty())
    }
}


pub fn run_server(cmd_handler: impl CommandHandler + 'static, config: ServerConfig) -> io::Result<()> {

    let cmd_handler = Arc::new(cmd_handler);

    println!("Server with ip address {} listening on port {}", config.address, config.port);

    TcpListener::bind(config.as_socket_addr())?
        .incoming()
        .filter_map(|socket|socket.ok())
        .for_each(|socket| {
            let cmd_handler = Arc::clone(&cmd_handler);
            thread::spawn(|| {
                let mut connection_data = ServerConnectionData {
                    socket,
                    user_id: None
                };
                if let Err(e) = handle_connection(cmd_handler, &mut connection_data) {
                    println!("Error occured {e}");
                    println!("User with id {:?} is disconnected", connection_data.user_id);
                }
            });
        });   

    Ok(())
}

fn handle_connection(cmd_handler: CommandHandlerRef, connection_data: &mut ServerConnectionData) -> io::Result<()> {

    println!("New connection from address {:?}", connection_data.socket.peer_addr());

    loop {
        let cmd = command_handler::fetch_command(&mut connection_data.socket);

        let mut server_response = match cmd {
            Ok(cmd) => handle_command(cmd, &cmd_handler, connection_data),
            Err(ServerError::MissingCommandData) => 
                    Notification::InvalidPayload.into(),
            Err(ServerError::CommandParsingError(e)) => 
                    handle_cmd_parsing_error(e).into(),
            Err(ServerError::IoError(e)) =>  Err(e)?
        };

        connection_data.socket.write(&[server_response.notification as u8])?;
        if let Some(data) = server_response.data_bytes.read_all() {
            connection_data.socket.write(data)?;
        }

        connection_data.socket.flush()?;

    }
}

fn handle_cmd_parsing_error(error: CommandParsingError) -> Notification {
    match error {
        CommandParsingError::UnknownCommand => Notification::UnknownCommand,
        CommandParsingError::InvalidPayload => Notification::InvalidPayload,
    }
}