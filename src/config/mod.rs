use std::{env, fmt::Display, str::FromStr};

use axum::http::HeaderValue;

const JWT_PLACEHOLDER: &str = "replace_with_a_secure_random_secret";
const JWT_MIN_LEN: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("invalid value for {name}: {reason}")]
    Invalid { name: &'static str, reason: String },
}

// No Debug derive: this struct holds secrets.
pub struct Config {
    pub database_url: String,
    pub db_max_connections: u32,
    pub server_host: String,
    pub server_port: u16,
    pub allowed_origins: Vec<HeaderValue>,
    pub jwt_secret: String,
    pub jwt_expiration_minutes: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let jwt_secret = required("JWT_SECRET")?;
        if jwt_secret == JWT_PLACEHOLDER || jwt_secret.len() < JWT_MIN_LEN {
            return Err(ConfigError::Invalid {
                name: "JWT_SECRET",
                reason: format!(
                    "must be a random string of at least {JWT_MIN_LEN} characters (try `openssl rand -hex 32`)"
                ),
            });
        }

        let jwt_expiration_minutes: i64 = parsed("JWT_EXPIRATION_MINUTES", 1440)?;
        if jwt_expiration_minutes <= 0 {
            return Err(ConfigError::Invalid {
                name: "JWT_EXPIRATION_MINUTES",
                reason: "must be greater than 0".into(),
            });
        }

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            db_max_connections: parsed("DB_MAX_CONNECTIONS", 10)?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            server_port: parsed("SERVER_PORT", 8080)?,
            allowed_origins: parse_origins(&required("FRONTEND_URL")?)?,
            jwt_secret,
            jwt_expiration_minutes,
        })
    }
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        _ => Err(ConfigError::Missing(name)),
    }
}

fn parsed<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: Display,
{
    match env::var(name) {
        Ok(v) if !v.trim().is_empty() => {
            v.trim().parse().map_err(|e: T::Err| ConfigError::Invalid {
                name,
                reason: e.to_string(),
            })
        }
        _ => Ok(default),
    }
}

fn parse_origins(raw: &str) -> Result<Vec<HeaderValue>, ConfigError> {
    let mut origins = Vec::new();
    for item in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if item == "*" {
            return Err(ConfigError::Invalid {
                name: "FRONTEND_URL",
                reason: "wildcard origin is not allowed".into(),
            });
        }
        let value = HeaderValue::from_str(item.trim_end_matches('/')).map_err(|e| {
            ConfigError::Invalid {
                name: "FRONTEND_URL",
                reason: format!("`{item}`: {e}"),
            }
        })?;
        origins.push(value);
    }
    if origins.is_empty() {
        return Err(ConfigError::Missing("FRONTEND_URL"));
    }
    Ok(origins)
}
