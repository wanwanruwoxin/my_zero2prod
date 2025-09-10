use std::time::Duration;

use sea_orm::ConnectOptions;
use secrecy::{ExposeSecret, SecretBox};
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretBox<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder();
    let base_path = std::env::current_dir().expect("当前目录获取失败");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("解析APP_ENVIRONMENT失败");
    println!("使用环境变量： {:?}", environment);

    let settings = settings
        .add_source(config::File::from(configuration_directory.join("base")))
        .add_source(config::File::from(
            configuration_directory.join(environment.as_str()),
        ))
        // 用环境变量的配置，来覆盖 configuration 的配置（环境变量的配置优先）
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()
        .unwrap();

    settings.try_deserialize()
}

#[derive(Debug)]
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(format!(
                "{}不是一个合法的环境变量, 使用`local` 或 `production`",
                other
            )),
        }
    }
}

impl DatabaseSettings {
    pub fn without_db(&self) -> ConnectOptions {
        let mut opt: ConnectOptions =
            ConnectOptions::new(self.connection_string_without_db().expose_secret());
        self.set_db_options(&mut opt);
        opt
    }

    pub fn with_db(&self) -> ConnectOptions {
        let mut opt: ConnectOptions = ConnectOptions::new(self.connection_string().expose_secret());
        self.set_db_options(&mut opt);
        opt
    }

    fn set_db_options(&self, opt: &mut ConnectOptions) {
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(2))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Info);
    }

    fn connection_string(&self) -> SecretBox<String> {
        SecretBox::init_with(|| {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port,
                self.database_name
            )
        })
    }

    fn connection_string_without_db(&self) -> SecretBox<String> {
        SecretBox::init_with(|| {
            format!(
                "postgres://{}:{}@{}:{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port
            )
        })
    }
}
