use std::fs;
use std::path::Path;

use arity_core::Orcid;
use arity_protocol::Person;
use serde::{Deserialize, Serialize};

use crate::error::{CliError, Result, io};

pub const CONFIG_FILE: &str = "arity.toml";

/// The settings of one data folder.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The person who checks reports and who is responsible for the
    /// submissions from this folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub person: Option<PersonConfig>,
    #[serde(default, skip_serializing_if = "HttpConfig::is_empty")]
    pub http: HttpConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonConfig {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orcid: Option<Orcid>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpConfig {
    /// An email address for the polite pools of Crossref. arity sends it
    /// only when this setting has a value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
}

impl HttpConfig {
    fn is_empty(&self) -> bool {
        self.contact.is_none()
    }
}

impl Config {
    pub fn load(directory: &Path) -> Result<Self> {
        let path = directory.join(CONFIG_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path).map_err(io(&path))?;
        toml::from_str(&text).map_err(|error| CliError::Config(error.to_string()))
    }

    pub fn save(&self, directory: &Path) -> Result<()> {
        let path = directory.join(CONFIG_FILE);
        let text = toml::to_string(self).map_err(|error| CliError::Config(error.to_string()))?;
        fs::write(&path, text).map_err(io(&path))
    }

    /// The person that the configuration names. A check needs one.
    pub fn person(&self) -> Result<Person> {
        let person = self.person.as_ref().ok_or_else(|| {
            CliError::Config(format!(
                "{CONFIG_FILE} names no person; run `arity init --name \"Your Name\"`"
            ))
        })?;
        Ok(Person {
            name: person.name.clone(),
            orcid: person.orcid.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_a_configuration() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(Config::load(directory.path()).unwrap(), Config::default());
        let config = Config {
            person: Some(PersonConfig {
                name: "A Person".to_owned(),
                orcid: Some("0000-0002-1825-0097".parse().unwrap()),
            }),
            http: HttpConfig::default(),
        };
        config.save(directory.path()).unwrap();
        assert_eq!(Config::load(directory.path()).unwrap(), config);
        assert_eq!(config.person().unwrap().name, "A Person");
    }

    #[test]
    fn a_check_needs_a_person() {
        assert!(matches!(
            Config::default().person(),
            Err(CliError::Config(_))
        ));
    }

    #[test]
    fn rejects_unknown_settings() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join(CONFIG_FILE), "colour = \"blue\"\n").unwrap();
        assert!(matches!(
            Config::load(directory.path()),
            Err(CliError::Config(_))
        ));
    }
}
