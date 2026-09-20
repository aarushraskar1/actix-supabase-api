use std::{env, num::ParseIntError};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub database_max_connections: u32,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingVariable(&'static str),
    #[error("invalid value for {variable}: {source}")]
    InvalidNumber {
        variable: &'static str,
        source: ParseIntError,
    },
    #[error(
        "SUPABASE_DATABASE_URL must be a PostgreSQL URL beginning with postgres:// or postgresql://"
    )]
    InvalidDatabaseUrl,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = env::var("SUPABASE_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .map_err(|_| ConfigError::MissingVariable("SUPABASE_DATABASE_URL or DATABASE_URL"))?;

        if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
            return Err(ConfigError::InvalidDatabaseUrl);
        }

        Ok(Self {
            database_url,
            host: optional("HOST", "0.0.0.0"),
            port: parse_with_alias("PORT", "SERVER_PORT", 8080)?,
            database_max_connections: parse("DATABASE_MAX_CONNECTIONS", 10)?,
        })
    }
}

fn optional(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn parse_with_alias<T>(
    primary: &'static str,
    alias: &'static str,
    default: T,
) -> Result<T, ConfigError>
where
    T: std::str::FromStr<Err = ParseIntError>,
{
    let value = match env::var(primary).or_else(|_| env::var(alias)) {
        Ok(value) => value,
        Err(_) => return Ok(default),
    };

    value.parse().map_err(|source| ConfigError::InvalidNumber {
        variable: primary,
        source,
    })
}

fn parse<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr<Err = ParseIntError>,
{
    env::var(name)
        .map(|value| {
            value.parse().map_err(|source| ConfigError::InvalidNumber {
                variable: name,
                source,
            })
        })
        .unwrap_or(Ok(default))
}
