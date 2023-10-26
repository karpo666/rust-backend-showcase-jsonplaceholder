use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JsonPlaceholder {
    pub url: String
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
    pub user: String,
    pub database_name: String,
    pub password: String
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub json_placeholder: JsonPlaceholder,
    pub database: Database
}

impl Configuration {

    /// Read configuration from file `config.toml`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to configuration file.
    pub fn read_from_config_file(path: &str) -> Result<Self, ConfigError> {
        // Read configuration from given path.
        let config_builder = Config::builder()
            .add_source(config::File::with_name(path))
            .build()?
        ;

        // Deserialize the result.
        config_builder.try_deserialize()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_from_config_file_failure() {
        assert!(Configuration::read_from_config_file("MADE_UP_PATH").is_err());
    }

    #[test]
    fn test_read_from_config_file_success() {
        let configuration_result = Configuration::read_from_config_file("resources/test/config");
        assert!(configuration_result.is_ok());
        let configuration = configuration_result.unwrap();
        assert_eq!("TEST_URL", configuration.json_placeholder.url);
        assert_eq!("TEST_URL", configuration.database.url);
    }
}