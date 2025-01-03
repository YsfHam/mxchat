use std::collections::HashMap;
use mxchat_core::auth::UserId;

pub struct MessagingInstance {
    pub text_to_send: String,
}

impl MessagingInstance {
    fn new() -> Self {
        Self {
            text_to_send: String::new(),
        }
    }
}

pub struct Messenger {
    intances: HashMap<UserId, MessagingInstance>,
}

impl Messenger {
    pub fn new() -> Self {
        Self {
            intances: HashMap::new()
        }
    }

    pub fn add_messsaging_instance(&mut self, user_id: UserId) {
        self.intances.insert(user_id, MessagingInstance::new());
    }

    pub fn get_messaging_instance(&mut self, user_id: UserId) -> Option<&mut MessagingInstance> {
        self.intances.get_mut(&user_id)
    }
}