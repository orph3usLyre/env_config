use env_config::EnvConfig;

#[derive(Debug, EnvConfig)]
struct DatabaseConfig {
    #[env_config(env = "DB_HOST")]
    host: String,
    #[env_config(env = "DB_PORT")]
    port: u16,
    #[env_config(env = "DB_NAME", default = "myapp")]
    database: String,
    #[env_config(env = "DB_SSL", default = "false")]
    ssl: bool,
}

#[derive(Debug, EnvConfig)]
struct RedisConfig {
    #[env_config(env = "REDIS_URL")]
    url: String,
    #[env_config(env = "REDIS_TIMEOUT", default = "5")]
    timeout: u64,
    #[env_config(env = "REDIS_MAX_CONNECTIONS", default = "10")]
    max_connections: u32,
}

#[derive(Debug, EnvConfig)]
struct LoggingConfig {
    #[env_config(env = "LOG_LEVEL", default = "info")]
    level: String,
    #[env_config(env = "LOG_FORMAT", default = "json")]
    format: String,
}

#[derive(Debug, EnvConfig)]
struct AppConfig {
    // Nested configurations - each loads its own env vars
    #[env_config(nested)]
    database: DatabaseConfig,

    #[env_config(nested)]
    redis: RedisConfig,

    #[env_config(nested)]
    logging: LoggingConfig,

    // Main app configuration
    #[env_config(env = "APP_NAME")]
    app_name: String,

    #[env_config(env = "APP_PORT", default = "8080")]
    port: u16,

    #[env_config(env = "APP_ENV", default = "development")]
    environment: String,
}

fn main() -> Result<(), env_config::EnvConfigError> {
    // Set some environment variables for demonstration
    //
    // # Safety
    // This example cannot run in parallel with other programs that set/remove ENV variables
    unsafe {
        // Database configuration
        std::env::set_var("DB_HOST", "localhost");
        std::env::set_var("DB_PORT", "5432");
        std::env::set_var("DB_NAME", "production_db");
        std::env::set_var("DB_SSL", "true");

        // Redis configuration
        std::env::set_var("REDIS_URL", "redis://localhost:6379");
        std::env::set_var("REDIS_TIMEOUT", "10");
        // REDIS_MAX_CONNECTIONS will use default value

        // Logging configuration
        std::env::set_var("LOG_LEVEL", "debug");
        // LOG_FORMAT will use default value

        // App configuration
        std::env::set_var("APP_NAME", "MyAwesomeApp");
        std::env::set_var("APP_PORT", "3000");
        std::env::set_var("APP_ENV", "production");
    }

    let config = AppConfig::from_env()?;

    println!("{:#?}", config);

    // Demonstrate accessing nested configuration
    println!(
        "Database connection: {}:{}",
        config.database.host, config.database.port
    );
    println!("Redis URL: {}", config.redis.url);
    println!("Log level: {}", config.logging.level);
    println!("App running on port: {}", config.port);

    // Verify the configuration was loaded correctly
    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.port, 5432);
    assert_eq!(config.database.database, "production_db");
    assert!(config.database.ssl);

    assert_eq!(config.redis.url, "redis://localhost:6379");
    assert_eq!(config.redis.timeout, 10);
    assert_eq!(config.redis.max_connections, 10); // default

    assert_eq!(config.logging.level, "debug");
    assert_eq!(config.logging.format, "json"); // default

    assert_eq!(config.app_name, "MyAwesomeApp");
    assert_eq!(config.port, 3000);
    assert_eq!(config.environment, "production");
    Ok(())
}
