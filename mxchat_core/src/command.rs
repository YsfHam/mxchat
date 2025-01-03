use crate::{auth::{UserConnectData, UserRegisterData}, io::BytesBuffer, utils};

#[derive(Debug)]
pub enum Command {
    Register(UserRegisterData),
    Connect(UserConnectData),
    RequestContact(String),
}

impl Command {
    pub fn from_bytes(bytes_buffer: &mut BytesBuffer) -> Result<Self, CommandParsingError> {
        let cmd_type = bytes_buffer.read_bytes(1)
            .ok_or(CommandParsingError::InvalidPayload)?[0];
        match cmd_type {
            0 => Self::parse_register_cmd(bytes_buffer),
            1 => Self::parse_connect_cmd(bytes_buffer),
            2 => Self::parse_request_contact_cmd(bytes_buffer),

            _ => Err(CommandParsingError::UnknownCommand)
        }
    }

    fn parse_register_cmd(bytes_buffer: &mut BytesBuffer) -> Result<Self, CommandParsingError> {
        bytes_buffer
            .read_all()
            .map(|bytes| String::from_utf8_lossy(bytes))
            .and_then(|data_str| UserRegisterData::new(&data_str))
            .map(|register_data| Command::Register(register_data))
            .ok_or(CommandParsingError::InvalidPayload)
    }

    fn parse_connect_cmd(bytes_buffer: &mut BytesBuffer) -> Result<Self, CommandParsingError> {

        bytes_buffer
            .read_all()
            .map(|bytes| String::from_utf8_lossy(bytes))
            .and_then(|data_str| UserConnectData::new(&data_str))
            .map(|connect_data| Command::Connect(connect_data))
            .ok_or(CommandParsingError::InvalidPayload)
    }

    fn parse_request_contact_cmd(bytes_buffer: &mut BytesBuffer) -> Result<Self, CommandParsingError> {
        bytes_buffer
            .read_all()
            .map(|bytes| String::from_utf8_lossy(bytes))
            .map(|username| username.to_string())
            .map(|username| Command::RequestContact(username))
            .ok_or(CommandParsingError::InvalidPayload)
    }


    // Serializing
    pub fn to_bytes(&self, bytes_buffer: &mut BytesBuffer) {
        
        match self {
            Command::Register(user_register_data) => {
                bytes_buffer.write_bytes(&[0]);
                
                utils::write_string_to_bytes_buffer(bytes_buffer, &user_register_data.to_string());
            }
            Command::Connect(user_connect_data) => {
                bytes_buffer.write_bytes(&[1]);

                utils::write_string_to_bytes_buffer(bytes_buffer, &user_connect_data.to_string());
            },
            Command::RequestContact(username) => {
                bytes_buffer.write_bytes(&[2]);
                utils::write_string_to_bytes_buffer(bytes_buffer, &username);
            }
        }
    }


}

#[derive(Debug)]
pub enum CommandParsingError {
    UnknownCommand,
    InvalidPayload
}