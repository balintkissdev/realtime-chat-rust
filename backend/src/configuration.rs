use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub backend: BackendConfig,
}

#[derive(Clone, Deserialize)]
pub struct BackendConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub rest_port: u16,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub ws_port: u16,
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "prod",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "prod" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `prod`.",
                other
            )),
        }
    }
}

pub fn get_config() -> Result<Config, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("../config");
    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("CHAT_APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse CHAT_APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let configuration = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        // Add settings from environment variables. E.g. `CHAT_APP_BACKEND__WS_PORT=9000`
        // Values can be injected without storing sensitive data in version control
        .add_source(
            config::Environment::with_prefix("CHAT_APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    configuration.try_deserialize::<Config>()
}
