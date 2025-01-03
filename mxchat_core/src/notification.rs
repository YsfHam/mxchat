#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Notification {
    UnknownCommand,
    InvalidPayload,

    // user notifs
    UserRegistred,
    UserAlreadyExist,

    UserConnected,
    UserIsAlreadyConnected,
    UserNotFound,
    UserPasswordIncorrect,

    ReceiveContactInfo,
}

impl Notification {
    pub fn has_payload(self) -> bool {
        match self {
            Notification::UnknownCommand => false,
            Notification::InvalidPayload => false,
            Notification::UserRegistred => false,
            Notification::UserAlreadyExist => false,
            Notification::UserPasswordIncorrect => false,
            Notification::UserIsAlreadyConnected => false,
            Notification::UserNotFound => false,

            
            Notification::UserConnected => true,
            Notification::ReceiveContactInfo => true,
        }
    }
}


impl TryFrom<u8> for Notification {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        [
            Self::UnknownCommand,
            Self::InvalidPayload,
            Self::UserRegistred,
            Self::UserAlreadyExist,
            Self::UserConnected,
            Self::UserIsAlreadyConnected,
            Self::UserNotFound,
            Self::UserPasswordIncorrect,
            Self::ReceiveContactInfo,
        ]
        .iter()
        .find(|variant| **variant as u8 == value)
        .map(|variant| *variant)
        .ok_or(())
    }
}

