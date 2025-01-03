use std::{collections::VecDeque, sync::RwLock};

use mxchat_core::{io::BytesBuffer, messaging::Contact, notification::Notification};

pub struct NotificationsQueue {
    notifications: RwLock<VecDeque<(Notification, BytesBuffer)>>
}

impl NotificationsQueue {
    pub fn new() -> Self {
        Self {
            notifications: RwLock::new(VecDeque::new())
        }
    }

    pub fn push_notification(&self, notification: Notification, payload: BytesBuffer) {
        self.notifications
            .write()
            .unwrap()
            .push_back((notification, payload));
    }

    pub fn pop_notification(&self) -> Option<(Notification, BytesBuffer)> {
        self.notifications
            .write()
            .unwrap()
            .pop_front()
    }
}

pub enum NotificationHandlerSignal {
    ContactReceived(Contact),
    ContactRetreivingFailed(String),
    None
}

pub struct ChatNotificationHandler;

impl ChatNotificationHandler {
    pub fn handle_notification(notification: Notification, mut payload: BytesBuffer) -> NotificationHandlerSignal {
        match notification {
            Notification::ReceiveContactInfo => {
                Contact::from_bytes(&mut payload)
                .inspect(|contact| println!("contact {:?}", contact))
                .map_or(
                    NotificationHandlerSignal::ContactRetreivingFailed(
                        "Error while retreiving user information".into()
                    ),
                    |contact| NotificationHandlerSignal::ContactReceived(contact)
                )
            },
            Notification::UserNotFound => {
                NotificationHandlerSignal::ContactRetreivingFailed(
                    "User not found".into()
                )
            }

            _ => NotificationHandlerSignal::None,
        }
    }
}

