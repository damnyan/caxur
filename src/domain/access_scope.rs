use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum AccessScope {
    #[serde(rename = "administrator")]
    Administrator,
}

impl AccessScope {
    pub fn all() -> Vec<AccessScope> {
        vec![AccessScope::Administrator]
    }
}

impl fmt::Display for AccessScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AccessScope::Administrator => "administrator",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for AccessScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "administrator" => Ok(AccessScope::Administrator),
            _ => Err(format!("Unknown access scope: {}", s)),
        }
    }
}
