use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};

use crate::schemas::Jwt;

#[derive(Deserialize, Serialize)]
pub struct Settings {
    pub database: DbSettings,
    pub app: AppSettings,
    pub avatar: AvatarSettings,
    pub auth: AuthSettings,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct AppSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Serialize)]
pub struct DbSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvatarSettings {
    pub base_url: String,
    pub default_img: String,
}

#[derive(Deserialize, Serialize)]
pub struct TokenConfig {
    pub key: String,
    pub exp: i64,
    pub maxage: i64,
}

#[derive(Deserialize, Serialize)]
pub struct AuthSettings {
    pub access: TokenConfig,
    pub refresh: TokenConfig,
}

pub struct AppState {
    pub connection: Pool<Postgres>,
    pub jwt: Jwt,
}

enum Environment {
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
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
    Use either `local` or `production`.",
                other
            )),
        }
    }
}

impl Settings {
    pub fn get_configuration() -> Result<Settings, config::ConfigError> {
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");

        let configuration_directory = base_path.join("configuration");

        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");

        let environment_filename = format!("{}.yaml", environment.as_str());
        let settings = config::Config::builder()
            .add_source(config::File::from(
                configuration_directory.join("base.yaml"),
            ))
            .add_source(config::File::from(
                configuration_directory.join(environment_filename),
            ))
            .build()?;

        settings.try_deserialize::<Settings>()
    }

    pub async fn create_app_state(&self) -> Result<AppState, sqlx::Error> {
        let connection = self.database.establish_connection().await?;
        Ok(AppState {
            connection,
            jwt: Jwt::new(&self.auth.access, &self.auth.refresh),
        })
    }
}

impl DbSettings {
    async fn establish_connection(&self) -> Result<PgPool, sqlx::Error> {
        let db_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        );

        tracing::info!("DB_URL: {}", db_url);
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(20))
            .connect_lazy(&db_url)
    }
}
