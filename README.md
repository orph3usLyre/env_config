# env_config

A simple Rust library for loading configuration from environment variables.

## Usage

### Derive Macro 

```rust
use env_config::EnvConfig;

#[derive(Debug, EnvConfig)]
struct AppConfig {
    // By default, we search for ENV variables using STRUCT_NAME_FIELD_NAME in SCREAMING_SNAKE_CASE.
    // `database_url` will be loaded from `APP_CONFIG_DATABASE_URL`
    database_url: String,                          // -> APP_CONFIG_DATABASE_URL (required)

    // if a default value is provided, that value is used as a fallback
    #[env_config(default = "8080")]
    port: u16,                                     // -> APP_CONFIG_PORT (with default)

    // if `optional` is set, it will use `None` if not found. This can only be implemented for `Option<T>`
    #[env_config(optional)]
    timeout: Option<u64>,                          // -> APP_CONFIG_TIMEOUT (optional)

    // custom ENV variable keys can be provided with `env = "CUSTOM_NAME"`
    #[env_config(env = "DEBUG_MODE")]
    debug: bool,                                   // -> DEBUG_MODE (custom name)

    // fields marked with `skip` will always use the `Default` impl for the type
    #[env_config(skip)]
    internal_state: String,                        // Skipped - uses Default::default()

    // fields marked with `parse_with = "my_fn_name"` will use the provided function to parse the env variable.
    // These functions must have the signature `(s: String) -> T`
    #[env_config(parse_with = "parse_point")] 
    position: Point,  // -> APP_CONFIG_POSITION (with custom parser)
}

// Use no_prefix to disable the struct name prefix
#[derive(Debug, EnvConfig)]
#[env_config(no_prefix)]
struct SimpleConfig {
    database_url: String,                          // -> DATABASE_URL (no prefix)
    port: u16,                                     // -> PORT (no prefix)
}

// Use custom prefix instead of struct name
#[derive(Debug, EnvConfig)]
#[env_config(prefix = "APP")]
struct CustomConfig {
    database_url: String,                          // -> APP_DATABASE_URL (custom prefix)
    port: u16,                                     // -> APP_PORT (custom prefix)
}

#[derive(Debug)]
struct Point { x: f64, y: f64 }

fn parse_point(s: String) -> Point {
    let parts: Vec<&str> = s.split(',').collect();
    Point {
        x: parts[0].parse().expect("Invalid x coordinate"),
        y: parts[1].parse().expect("Invalid y coordinate"),
    }
}

fn main() -> Result<(), env_config::EnvConfigError> {
    let config = AppConfig::from_env()?;
    println!("Config: {:?}", config);
    Ok(())
}
```

### Nested Configuration

You can compose complex configs from smaller structs using the `nested` attribute:

```rust
use env_config::EnvConfig;

#[derive(Debug, EnvConfig)]
struct DatabaseConfig {
    #[env_config(env = "DB_HOST")]
    host: String,                               // -> DB_HOST
    #[env_config(env = "DB_PORT")]
    port: u16,                                  // -> DB_PORT
    #[env_config(env = "DB_NAME", default = "myapp")]
    database: String,                           // -> DB_NAME (with default)
}

#[derive(Debug, EnvConfig)]
struct RedisConfig {
    #[env_config(env = "REDIS_URL")]
    url: String,                                // -> REDIS_URL
    #[env_config(env = "REDIS_TIMEOUT", default = "5")]
    timeout: u64,                               // -> REDIS_TIMEOUT (with default)
}

#[derive(Debug, EnvConfig)]
struct AppConfig {
    #[env_config(nested)]
    database: DatabaseConfig,                   // Loads from DB_* env vars
    
    #[env_config(nested)]
    redis: RedisConfig,                         // Loads from REDIS_* env vars
    
    #[env_config(env = "APP_NAME")]
    app_name: String,                           // -> APP_NAME
    
    #[env_config(default = "info")]
    log_level: String,                          // -> LOG_LEVEL (with default)
}

fn main() -> Result<(), env_config::EnvConfigError> {
    let config = AppConfig::from_env()?;
    println!("Config: {:#?}", config);
    Ok(())
}
```

### Manual Implementation

```rust
use env_config::{EnvConfig, EnvConfigError, env_var, env_var_or, env_var_optional};

#[derive(Debug)]
struct AppConfig {
    database_url: String,
    port: u16,
    debug: bool,
    timeout: Option<u64>,
}

impl EnvConfig for AppConfig {
    type Error = EnvConfigError;
    
    fn from_env() -> Result<Self, Self::Error> {
        Ok(AppConfig {
            database_url: env_var("DATABASE_URL")?,    // Required
            port: env_var_or("PORT", 8080)?,           // Default to 8080
            debug: env_var_or("DEBUG", false)?,        // Default to false
            timeout: env_var_optional("TIMEOUT")?,     // Optional
        })
    }
}
```

## Derive Macro Attributes

**Struct-level attributes:**
- **`#[env_config(no_prefix)]`**: Don't use struct name as prefix for field names
- **`#[env_config(prefix = "PREFIX")]`**: Use custom prefix instead of struct name

**Field-level attributes:**
- **No attribute**: Field name is prefixed with struct name and converted to UPPER_SNAKE_CASE for env var name
- **`#[env_config(env = "VAR_NAME")]`**: Use custom environment variable name (overrides prefix)
- **`#[env_config(default = "value")]`**: Provide default value if env var not set
- **`#[env_config(optional)]`**: Make field optional (must be `Option<T>`)
- **`#[env_config(skip)]`**: Skip this field (uses `Default::default()`)
- **`#[env_config(parse_with = "function_name")]`**: Use custom parser function (takes `String`, returns `T`)
- **`#[env_config(nested)]`**: Treat field as nested EnvConfig struct (calls `T::from_env()`)

## Helper Functions

- **`env_var<T>(name)`**: Load a required environment variable
- **`env_var_optional<T>(name)`**: Load an optional environment variable (returns `Option<T>`)
- **`env_var_or<T>(name, default)`**: Load an environment variable with a default value
- **`env_var_with_parser<T, F>(name, parser)`**: Load a required environment variable with custom parser
- **`env_var_optional_with_parser<T, F>(name, parser)`**: Load an optional environment variable with custom parser

## Error variants

- `EnvConfigError::Missing(String)`: Environment variable is not set (Key)
- `EnvConfigError::Parse(String, String)`: Failed to parse value (Key, Value)


## License

Apache License 2.0
