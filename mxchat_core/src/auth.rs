use crate::{io::BytesBuffer, utils::{bytes_as_u32, u32_as_bytes}};


#[derive(Debug, Clone)]
pub struct UserRegisterData {
    pub username: String,
    pub nickname: String,
    pub password: String
}

impl UserRegisterData {
    pub fn new(data_str: &str) -> Option<Self> {
        let mut iter = data_str.split(';');
        let username = iter.next()?.to_string();
        let nickname = iter.next()?.to_string();
        let password = iter.next()?.to_string();

        Some(Self {
            username,
            nickname,
            password
        })
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(
            self.username.len() + 1 + 
            self.nickname.len() + 1 + 
            self.password.len()
        );

        result.push_str(&self.username);
        result.push(';');
        result.push_str(&self.nickname);
        result.push(';');
        result.push_str(&self.password);

        result
    }
}

#[derive(Debug, Clone)]
pub struct UserConnectData {
    pub username: String,
    pub password: String,
}

impl UserConnectData {
    pub fn new(data_str: &str) -> Option<Self> {
        let mut iter = data_str.split(';');
        let username = iter.next()?.to_string();
        let password = iter.next()?.to_string();

        Some(Self {
            username,
            password,
        })
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(
            self.username.len() + 1 + 
            self.password.len()
        );

        result.push_str(&self.username);
        result.push(';');
        result.push_str(&self.password);

        result
    }
}

type UserIdInner = u32;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct UserId(UserIdInner);

impl UserId {

    pub const fn size() -> usize {
        std::mem::size_of::<UserIdInner>()
    }

    pub fn from_bytes(bytes: &[u8; Self::size()]) -> Self {
        Self::new(bytes_as_u32(bytes))
    }

    pub fn to_bytes(self) -> [u8; Self::size()] {
        u32_as_bytes(self.get())
    }

    pub fn new(value: UserIdInner) -> Self {
        Self(value)
    }

    pub fn get(self) -> UserIdInner {
        self.0
    }
}

#[derive(Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub nickname: String,
}

impl User {
    pub fn from_bytes(bytes_buffer: &mut BytesBuffer) -> Option<Self> {
        let user_id = bytes_buffer.read_bytes(UserId::size())?;

        let id = UserId::from_bytes(&[
            user_id[0],
            user_id[1],
            user_id[2],
            user_id[3]
        ]);

        let data = String::from_utf8_lossy(bytes_buffer.read_all()?);

        let mut iter = data.split(';');
        let username = iter.next()?.to_string();
        let nickname = iter.next()?.to_string();

        Some(User {
            id,
            username,
            nickname
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            UserId::size() + 
            self.username.len() + 1 + 
            self.nickname.len()
        );

        result.extend_from_slice(&self.id.to_bytes());
        result.extend_from_slice(self.username.as_bytes());
        result.extend_from_slice(b";");
        result.extend_from_slice(self.nickname.as_bytes());

        result
    }
}