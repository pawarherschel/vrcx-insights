use std::str::FromStr;

#[derive(Debug, Clone, Copy, sqlx::Type, Default, PartialEq, Eq, Hash)]
pub enum Regions {
    #[default]
    Other,
    USWest,
    US,
    USEast,
    Europe,
    Japan,
}

impl From<&str> for Regions {
    #[inline]
    fn from(value: &str) -> Self {
        let value = value.to_lowercase();
        Self::from(value)
    }
}

#[allow(clippy::fallible_impl_from)] // ~kat
impl From<String> for Regions {
    #[inline]
    fn from(value: String) -> Self {
        let value = value.to_lowercase();
        match value.as_str() {
            "uswest" | "usw" | "us w" | "us_w" | "uw" => Self::USWest,
            "us" => Self::US,
            "useast" | "use" | "us e" | "us_e" | "ue" => Self::USEast,
            "europe" | "eu" => Self::Europe,
            "japan" | "jp" => Self::Japan,

            _ => panic!("Unknown region: {value}"),
        }
    }
}

impl FromStr for Regions {
    type Err = std::string::ParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}
