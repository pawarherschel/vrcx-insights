use std::str::FromStr;

#[derive(Debug, Clone, Copy, sqlx::Type, Default)]
pub enum JoinLeaveEvent {
    Join,
    Leave,
    #[default]
    Other,
}

impl From<&str> for JoinLeaveEvent {
    fn from(value: &str) -> Self {
        let value = value.to_lowercase();
        Self::from(value)
    }
}

impl From<String> for JoinLeaveEvent {
    fn from(value: String) -> Self {
        let value = value.to_lowercase();
        match value.as_str() {
            "join" => JoinLeaveEvent::Join,
            "leave" => JoinLeaveEvent::Leave,

            "joins" => JoinLeaveEvent::Join,
            "leaves" => JoinLeaveEvent::Leave,

            "joined" => JoinLeaveEvent::Join,
            "left" => JoinLeaveEvent::Leave,

            "onplayerjoined" => JoinLeaveEvent::Join,
            "onplayerleft" => JoinLeaveEvent::Leave,

            _ => panic!("Unknown join/leave event: {}", value),
        }
    }
}

impl FromStr for JoinLeaveEvent {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}
