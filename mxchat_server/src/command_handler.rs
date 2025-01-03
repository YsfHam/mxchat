use std::{io::Read, net::TcpStream, sync::Arc};

use mxchat_core::{auth::{UserConnectData, UserRegisterData}, command::Command, io::BytesBuffer, utils::bytes_as_u32};

use crate::server::{ServerConnectionData, ServerError, ServerResponse};

pub trait CommandHandler: Send + Sync {
    fn handle_register_cmd(&self, user_register_data: UserRegisterData) -> ServerResponse;
    fn handle_connect_cmd(&self, user_connect_data: UserConnectData, connection_data: &mut ServerConnectionData) -> ServerResponse;
    fn handle_request_contact_cmd(&self, username: &str) -> ServerResponse;
}

pub type CommandHandlerRef = Arc<dyn CommandHandler>;


pub fn fetch_command(socket: &mut TcpStream) -> Result<Command, ServerError> {
    let mut data_bytes = read_command_data(socket)?;

    Command::from_bytes(&mut data_bytes)
        .map_err(|e| ServerError::CommandParsingError(e))
}

pub fn handle_command(cmd: Command, command_handler: &CommandHandlerRef, connection_data: &mut ServerConnectionData) -> ServerResponse {
    match cmd {
        Command::Register(user_register_data) => command_handler.handle_register_cmd(user_register_data),
        Command::Connect(user_connect_data) => command_handler.handle_connect_cmd(user_connect_data, connection_data),
        Command::RequestContact(username) => command_handler.handle_request_contact_cmd(&username),
    }
}

fn read_command_data(socket: &mut TcpStream) -> Result<BytesBuffer, ServerError> {

    let mut byte = [0u8; 1];
    if socket.read(&mut byte)? == 0 {
        return Err(ServerError::MissingCommandData);
    }

    let mut length_bytes = [0u8; 4];
    if socket.read(&mut length_bytes)? == 0 {
        return Err(ServerError::MissingCommandData);
    }

    let payload_length = bytes_as_u32(&length_bytes);

    let mut payload_bytes = vec![0u8; payload_length as usize];
    if socket.read(&mut payload_bytes)? == 0 {
        return Err(ServerError::MissingCommandData);
    }

    let mut bytes_buffer = BytesBuffer::empty();
    bytes_buffer.write_bytes(&byte);
    bytes_buffer.write_bytes(&payload_bytes);

    Ok(bytes_buffer)
}