# env_config

A rust derive macro library for loading configuration from environment variables.

## Usage

```rust
use env_config::EnvConfig;
use std::time::Duration;

#[derive(Debug, EnvConfig)]
struct AppConfig {
    // By default, we search for ENV variables using STRUCT_NAME_FIELD_NAME in SCREAMING_SNAKE_CASE.
    // `database_url` will be loaded from `APP_CONFIG_DATABASE_URL`
    url: String, // -> APP_CONFIG_URL (required)

    // if a default value is provided, that value is used as a fallback
    #[env_config(default = "8080")]
    port: u16, // -> APP_CONFIG_PORT (with default)

    timeout: Option<u64>, // -> APP_CONFIG_TIMEOUT (optional)

    // custom ENV variable keys can be provided with `env = "CUSTOM_NAME"`
    #[env_config(env = "DEBUG_MODE")]
    debug: bool, // -> DEBUG_MODE (custom name)

    // fields marked with `skip` will always use the `Default` impl for the type
    #[env_config(skip)]
    internal_state: Option<String>, // Skipped - uses Default::default()

    // fields marked with `parse_with = "my_fn_name"` will use the provided function to parse the env variable.
    // These functions must have the signature `fn(String) -> T`
    #[env_config(parse_with = "parse_point")]
    position: Point, // -> APP_CONFIG_POSITION (with custom parser)

    // fields marked with `parse_with = "my_fn_name"` can also be optional
    #[env_config(parse_with = "parse_timeout_with_default")]
    timeout_duration: Option<Duration>, // -> APP_CONFIG_TIMEOUT_DURATION (with custom parser that provides defaults)

    #[env_config(nested)]
    db_config: DatabaseConfig,

    #[env_config(nested)]
    redis_config: RedisConfig,
}

#[derive(Debug, EnvConfig)]
// Use no_prefix to disable the struct name prefix
#[env_config(no_prefix)]
struct DatabaseConfig {
    postgres_url: String, // -> POSTGRES_URL
    #[env_config(env = "DB_NAME", default = "mydb")]
    database: String, // -> DB_NAME (with default)
}

#[derive(Debug, EnvConfig)]
// Use custom prefix instead of struct name
#[env_config(prefix = "REDIS")]
struct RedisConfig {
    url: String, // -> REDIS_URL
    #[env_config(default = "5")]
    cache_timeout: u64, // -> REDIS_CACHE_TIMEOUT (with default)
}

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

fn parse_point(s: String) -> Point {
    let (x, y) = s.split_once(',').expect("Invalid format");
    Point {
        x: x.trim().parse().expect("Invalid x coordinate"),
        y: y.trim().parse().expect("Invalid y coordinate"),
    }
}

fn parse_timeout_with_default(s: String) -> Duration {
    Duration::from_secs(s.parse::<u64>().unwrap_or(100))
}

fn main() -> Result<(), env_config::EnvConfigError> {
    // Set some environment variables for demonstration
    //
    // # Safety
    // This example cannot run in parallel with other programs that set/remove ENV variables
    unsafe {
        std::env::set_var("APP_CONFIG_URL", "0.0.0.0:8080");
        std::env::set_var("APP_CONFIG_TIMEOUT", "42");
        std::env::set_var("DEBUG_MODE", "true");
        std::env::set_var("APP_CONFIG_POSITION", "42.43, 893.2123");
        std::env::set_var("APP_CONFIG_TIMEOUT_DURATION", "243");
        std::env::set_var("POSTGRES_URL", "postgres://postgres:postgres@0.0.0.0:5432");
        std::env::set_var("REDIS_URL", "redis://localhost:6379");
    }
    let config = AppConfig::from_env()?;
    println!("AppConfig: {config:#?}");

    Ok(())
}
```

## Derive Macro Attributes

**Struct attributes:**
- **`#[env_config(no_prefix)]`**: Don't use struct name as prefix for field names
- **`#[env_config(prefix = "PREFIX")]`**: Use custom prefix instead of struct name

**Field attributes:**
- **No attribute**: Field name is prefixed with struct name and converted to UPPER_SNAKE_CASE for env var name
- **`#[env_config(env = "VAR_NAME")]`**: Use custom environment variable name (overrides prefix)
- **`#[env_config(default = "value")]`**: Provide default value if env var not set
- **`#[env_config(skip)]`**: Skip this field (uses `Default::default()`)
- **`#[env_config(parse_with = "function_name")]`**: Use custom parser function (takes `String`, returns `T`)
- **`#[env_config(nested)]`**: Treat field as nested EnvConfig struct (calls `T::from_env()`)

## Error variants

- `EnvConfigError::Missing(String)`: Environment variable is not set (Key)
- `EnvConfigError::Parse(String, String)`: Failed to parse value (Key, Value)


## License

Apache License 2.0
