use std::{collections::HashMap, sync::atomic::{AtomicU32, Ordering}};

use mxchat_core::auth::{User, UserId};

pub struct UserData {
    pub user: User,
    pub password: String,
}

pub trait UserRepository: Sync + Send {
    fn add_user(&mut self, user: UserData);
    fn find_user_with_username(&self, username: &str) -> Option<&UserData>;
}

pub struct InMemoryUserRepository {
    users: Vec<UserData>,
    users_ids: HashMap<UserId, usize>,
    users_usernames: HashMap<String, usize>
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            users_ids: HashMap::new(),
            users_usernames: HashMap::new(),
        }
    }
}

impl UserRepository for InMemoryUserRepository {
    fn add_user(&mut self, user: UserData) {
        let index = self.users.len();

        self.users_ids.insert(user.user.id, index);
        self.users_usernames.insert(user.user.username.clone(), index);

        self.users.push(user);
    }
    
    fn find_user_with_username(&self, username: &str) -> Option<&UserData> {
        self
            .users_usernames
            .get(username)
            .and_then(|index| self.users.get(*index))
    }
}

pub struct UserIdGenerator {
    current_id: AtomicU32,
}

impl UserIdGenerator {
    pub fn new() -> Self {
        Self {
            current_id: AtomicU32::new(0),
        }
    }

    pub fn next_id(&self) -> UserId {
       let id = self.current_id.fetch_add(1, Ordering::Relaxed);

        UserId::new(id)
    }
}

