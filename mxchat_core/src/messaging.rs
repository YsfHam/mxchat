use crate::{auth::UserId, io::BytesBuffer};

#[derive(Debug)]
pub struct Contact {
    pub id: UserId,
    pub nickname: String
}

impl Contact {
    pub fn from_bytes(bytes_buffer: &mut BytesBuffer) -> Option<Self> {
        let user_id_bytes = bytes_buffer.read_bytes(UserId::size())?;
        let id = UserId::from_bytes(&[
            user_id_bytes[0],
            user_id_bytes[1],
            user_id_bytes[2],
            user_id_bytes[3],
        ]);

        let nickname = String::from_utf8_lossy(bytes_buffer.read_all()?).to_string();

        Some(Self {
            id,
            nickname
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&self.id.to_bytes());
        result.extend_from_slice(self.nickname.as_bytes());

        result
    }
}