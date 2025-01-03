use std::{io::{self, Read, Write}, net::TcpStream};

use mxchat_core::{auth::{UserConnectData, UserRegisterData}, command::Command, io::BytesBuffer, notification::Notification, utils::bytes_as_u32};

pub fn send_register_cmd(socket: &mut TcpStream, user_register_data: UserRegisterData) -> io::Result<()> {
    let cmd = Command::Register(user_register_data);

    send_cmd(socket, cmd)
}

pub fn send_connect_cmd(socket: &mut TcpStream, user_connect_data: UserConnectData) -> io::Result<()> {
    let cmd = Command::Connect(user_connect_data);

    send_cmd(socket, cmd)
}

pub fn send_request_contact_cmd(socket: &mut TcpStream, username: String) -> io::Result<()> {
    let cmd = Command::RequestContact(username);

    send_cmd(socket, cmd)
}

pub fn read_notification(socket: &mut TcpStream) -> io::Result<Notification> {
    let mut byte = [0u8; 1];
    if socket.read(&mut byte)? == 0 {
        return Err(std::io::ErrorKind::InvalidData.into());
    }


    match byte[0].try_into() {
        Err(_) => Err(std::io::ErrorKind::InvalidData.into()),
        Ok(notif) => Ok(notif)
    }

}

pub fn read_notification_payload(socket: &mut TcpStream) -> io::Result<BytesBuffer> {
    let mut payload_length = [0u8; 4];
    socket.read(&mut payload_length)?;

    let payload_length = bytes_as_u32(&payload_length);

    let mut payload = vec![0u8; payload_length as usize];

    socket.read(&mut payload)?;

    Ok(BytesBuffer::from_bytes(payload))
}

fn send_cmd(socket: &mut TcpStream, cmd: Command) -> io::Result<()> {

    let mut bytes_buffer = BytesBuffer::empty();
    cmd.to_bytes(&mut bytes_buffer);

    socket.write_all(bytes_buffer.read_all().unwrap())?;
    socket.flush()?;

    Ok(())
}