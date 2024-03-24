use std::str::FromStr;

use crate::zaphkiel::group_access_type::GroupAccessType;
use crate::zaphkiel::world_regions::Regions;

#[derive(Debug, Clone, Default, sqlx::Type, PartialEq, Eq, Hash)]
pub struct WorldInstance {
    pub world_id: String,
    pub instance_id: String,
    pub nonce: Option<String>,
    pub hidden: Option<String>,
    pub private: Option<String>,
    pub region: Option<Regions>,
    pub friends: Option<String>,
    pub group: Option<String>,
    pub group_access_type: Option<GroupAccessType>,
}

impl WorldInstance {
    /// Create a new `WorldInstance` with default values as specified in the `Default` trait.
    #[allow(dead_code)]
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    #[inline]
    pub fn get_prefix(&self) -> String {
        format!("{}:{}", self.world_id, self.instance_id)
    }
}

/// A struct representing a world instance parse error.
///
/// Valid parse errors:
///
/// - `Empty`: The string is empty.
/// - `InvalidFormat`: The string is not in the correct format.
/// - `InvalidWorldId`: The world id is invalid.
/// - `InvalidInstanceId`: The instance id is invalid.
/// - `InvalidOptionalField`: The optional field is invalid.
/// - `Other`: Other errors.
#[derive(Debug, Clone, sqlx::Type, Default, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)] // I want it like that ~kat
pub enum WorldInstanceParseError {
    Empty,
    InvalidFormat,
    InvalidWorldId,
    InvalidInstanceId,
    InvalidOptionalField,
    #[default]
    Other,
}

impl FromStr for WorldInstance {
    type Err = WorldInstanceParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(WorldInstanceParseError::Empty);
        }

        let mut ret = Self::new();

        let world_id = s;
        let parts = world_id.split(':').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(WorldInstanceParseError::InvalidFormat);
        }

        if parts[0].is_empty() {
            return Err(WorldInstanceParseError::InvalidWorldId);
        }
        ret.world_id = parts[0].to_string();

        let parts = parts[1].split('~').collect::<Vec<_>>();
        if parts[0].is_empty() {
            return Err(WorldInstanceParseError::InvalidInstanceId);
        }
        ret.instance_id = parts[0].to_string();

        for part in parts {
            let parts = part.split('(').collect::<Vec<_>>();
            let key = parts[0];
            if parts.len() < 2 {
                continue;
            }
            let value = parts[1].split(')').collect::<Vec<_>>()[0].to_string();

            match key {
                "nonce" => ret.nonce = Some(value),
                "hidden" => ret.hidden = Some(value),
                "private" => ret.private = Some(value),
                "region" => ret.region = Some(value.into()),
                "friends" => ret.friends = Some(value),
                "group" => ret.group = Some(value),
                "groupAccessType" => ret.group_access_type = Some(value.into()),
                _ => panic!("Unknown key: {key}, {part}"),
            }
        }

        Ok(ret)
    }
}

#[allow(clippy::fallible_impl_from)] // I want it like that ~kat
impl From<&str> for WorldInstance {
    #[inline]
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap()
    }
}

#[allow(clippy::fallible_impl_from)] // I want it like that ~kat
impl From<String> for WorldInstance {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::zaphkiel::world_instance::{WorldInstance, WorldInstanceParseError};
    use crate::zaphkiel::world_regions::Regions;

    #[test]
    fn test_parse_world_instance() {
        let world_instance_str = "world_id:instance_id~region(EU)";
        let expected_world_instance = world_instance_data();
        let actual_world_instance = WorldInstance::from_str(world_instance_str).unwrap();
        assert_eq!(actual_world_instance, expected_world_instance);
    }

    fn world_instance_data() -> WorldInstance {
        WorldInstance {
            world_id: "world_id".to_string(),
            instance_id: "instance_id".to_string(),
            nonce: None,
            hidden: None,
            private: None,
            region: Some(Regions::Europe),
            friends: None,
            group: None,
            group_access_type: None,
        }
    }

    #[test]
    fn test_parse_world_instance_invalid_format() {
        let world_instance_str = "invalid_format";
        let actual_result = WorldInstance::from_str(world_instance_str);
        assert!(actual_result.is_err());
        assert_eq!(
            actual_result.unwrap_err(),
            WorldInstanceParseError::InvalidFormat
        );
    }

    #[test]
    fn test_parse_world_instance_invalid_world_id() {
        let world_instance_str = ":instance_id~region(EU)";
        let actual_result = WorldInstance::from_str(world_instance_str);
        assert!(actual_result.is_err());
        assert_eq!(
            actual_result.unwrap_err(),
            WorldInstanceParseError::InvalidWorldId
        );
    }

    #[test]
    fn test_parse_world_instance_invalid_instance_id() {
        let world_instance_str = "world_id:~region(EU)";
        let actual_result = WorldInstance::from_str(world_instance_str);
        assert!(actual_result.is_err());
        assert_eq!(
            actual_result.unwrap_err(),
            WorldInstanceParseError::InvalidInstanceId
        );
    }

    #[test]
    #[should_panic(expected = "unknown key in world instance string")]
    fn test_parse_world_instance_unknown_key() {
        let world_instance_str = "world_id:instance_id~unknown_key(value)";
        let actual_result = WorldInstance::from_str(world_instance_str);
        assert!(actual_result.is_err());
        assert_eq!(actual_result.unwrap_err(), WorldInstanceParseError::Other);
    }

    #[test]
    fn test_from_str_for_world_instance_empty_input() {
        let world_instance_str = "";
        let actual_result = WorldInstance::from_str(world_instance_str);
        assert!(actual_result.is_err());
        assert_eq!(actual_result.unwrap_err(), WorldInstanceParseError::Empty);
    }

    #[test]
    fn test_from_str_for_world_instance_from_string() {
        let world_instance_str = "world_id:instance_id~region(EU)";
        let expected_world_instance = world_instance_data();
        let actual_world_instance = WorldInstance::from(world_instance_str.to_string());
        assert_eq!(actual_world_instance, expected_world_instance);
    }

    #[test]
    fn test_from_str_for_world_instance_from_str() {
        let world_instance_str = "world_id:instance_id~region(EU)";
        let expected_world_instance = world_instance_data();
        let actual_world_instance = WorldInstance::from(world_instance_str);
        assert_eq!(actual_world_instance, expected_world_instance);
    }
}
