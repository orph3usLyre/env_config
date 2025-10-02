use std::str::FromStr;

// Re-export the derive macro
pub use env_config_derive::EnvConfig;

/// Trait for loading configuration from environment variables.
///
/// This trait provides a simple interface for loading configuration from environment variables.
/// You can implement this trait manually using the helper functions, or use the derive macro
/// for automatic field mapping.
///
/// # Manual Implementation Example
///
/// ```rust
/// use env_config::{EnvConfig, EnvConfigError, env_var, env_var_or, env_var_optional};
///
/// #[derive(Debug)]
/// struct AppConfig {
///     database_url: String,
///     port: u16,
///     debug: bool,
///     timeout: Option<u64>,
/// }
///
/// impl EnvConfig for AppConfig {
///     type Error = EnvConfigError;
///     
///     fn from_env() -> Result<Self, Self::Error> {
///         Ok(AppConfig {
///             database_url: env_var("DATABASE_URL")?,           // Required
///             port: env_var_or("PORT", 8080)?,                 // Default to 8080
///             debug: env_var_or("DEBUG", false)?,              // Default to false
///             timeout: env_var_optional("TIMEOUT")?,           // Optional
///         })
///     }
/// }
/// ```
///
/// # Derive Macro Example
///
/// ```rust
/// use env_config::*;
///
/// #[derive(Debug, EnvConfig)]
/// struct AppConfig {
///     database_url: String,                          // -> APP_CONFIG_DATABASE_URL (required)
///     #[env_config(optional)]
///     timeout: Option<u64>,                          // -> APP_CONFIG_TIMEOUT (optional)
///     #[env_config(env = "DEBUG_MODE")]
///     debug: bool,                                   // -> DEBUG_MODE (custom name)
///     #[env_config(skip)]
///     internal_state: String,                        // Skipped - uses Default::default()
/// }
///
/// // Use no_prefix to disable the struct name prefix
/// #[derive(Debug, EnvConfig)]
/// #[env_config(no_prefix)]
/// struct SimpleConfig {
///     database_url: String,                          // -> DATABASE_URL (no prefix)
/// }
///
/// // Use custom prefix instead of struct name
/// #[derive(Debug, EnvConfig)]
/// #[env_config(prefix = "APP")]
/// struct CustomConfig {
///     database_url: String,                          // -> APP_DATABASE_URL (custom prefix)
/// }
/// ```
///
/// # Derive Macro Attributes
///
/// **Struct-level attributes:**
/// - **`#[env_config(no_prefix)]`**: Don't use struct name as prefix for field names
/// - **`#[env_config(prefix = "PREFIX")]`**: Use custom prefix instead of struct name
///
/// **Field-level attributes:**
/// - **No attribute**: Field name is prefixed with struct name and converted to UPPER_SNAKE_CASE
/// - **`#[env_config(env = "VAR_NAME")]`**: Use custom environment variable name
/// - **`#[env_config(default = "value")]`**: Provide default value if env var not set
/// - **`#[env_config(skip)]`**: Skip this field (must implement `Default`)
/// - **`#[env_config(parse_with = "function_name")]`**: Use custom parser function (takes `String`, returns `T`)
/// - **`#[env_config(nested)]`**: Treat field as nested EnvConfig struct (calls `T::from_env()`)
pub trait EnvConfig: Sized {
    /// Error type returned by `from_env()`.
    type Error;

    /// Load configuration from environment variables.
    fn from_env() -> Result<Self, Self::Error>;
}

/// Error type for environment configuration loading.
#[derive(Debug, thiserror::Error)]
pub enum EnvConfigError {
    /// Environment variable is missing.
    #[error("Missing environment variable: `{0}`")]
    Missing(String),
    /// Failed to parse environment variable value.
    #[error("Failed to parse environment variable: '{0}': {1}")]
    Parse(String, String),
}

// Helper functions for implementing the trait
/// Load a required environment variable and parse it to the target type.
/// Fails if the variable is not set or cannot be parsed.
pub fn env_var<T>(name: &str) -> Result<T, EnvConfigError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let value = std::env::var(name).map_err(|_| EnvConfigError::Missing(name.to_string()))?;
    value
        .parse::<T>()
        .map_err(|e| EnvConfigError::Parse(name.to_string(), e.to_string()))
}

/// Load an optional environment variable and parse it to the target type.
/// Returns `None` if the variable is not set.
pub fn env_var_optional<T>(name: &str) -> Result<Option<T>, EnvConfigError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match std::env::var(name) {
        Ok(value) => value
            .parse::<T>()
            .map(Some)
            .map_err(|e| EnvConfigError::Parse(name.to_string(), e.to_string())),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(EnvConfigError::Parse(
            name.to_string(),
            "Invalid Unicode".to_string(),
        )),
    }
}

/// Load an environment variable with a default value if not present.
pub fn env_var_or<T>(name: &str, default: T) -> Result<T, EnvConfigError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match env_var_optional(name)? {
        Some(value) => Ok(value),
        None => Ok(default),
    }
}

/// Load an environment variable with a string default that gets parsed if env var not present.
pub fn env_var_or_parse<T>(name: &str, default: &str) -> Result<T, EnvConfigError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match std::env::var(name) {
        Ok(value) => value
            .parse::<T>()
            .map_err(|e| EnvConfigError::Parse(name.to_string(), e.to_string())),
        Err(std::env::VarError::NotPresent) => default
            .parse::<T>()
            .map_err(|e| EnvConfigError::Parse(format!("default for {}", name), e.to_string())),
        Err(std::env::VarError::NotUnicode(_)) => Err(EnvConfigError::Parse(
            name.to_string(),
            "Invalid Unicode".to_string(),
        )),
    }
}

/// Load a required environment variable and parse it using a custom parser function.
/// The parser function should take a String and return the target type T.
/// Any panics or errors from the parser function will bubble up naturally.
pub fn env_var_with_parser<T, F>(name: &str, parser: F) -> Result<T, EnvConfigError>
where
    F: FnOnce(String) -> T,
{
    let value = std::env::var(name).map_err(|_| EnvConfigError::Missing(name.to_string()))?;
    Ok(parser(value))
}

/// Load an optional environment variable and parse it using a custom parser function.
/// Returns None if the variable is not set.
/// The parser function should take a String and return the target type T.
/// Any panics or errors from the parser function will bubble up naturally.
pub fn env_var_optional_with_parser<T, F>(
    name: &str,
    parser: F,
) -> Result<Option<T>, EnvConfigError>
where
    F: FnOnce(String) -> T,
{
    match std::env::var(name) {
        Ok(value) => Ok(Some(parser(value))),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(EnvConfigError::Parse(
            name.to_string(),
            "Invalid Unicode".to_string(),
        )),
    }
}
