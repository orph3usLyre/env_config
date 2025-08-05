#![allow(dead_code)]

use env_config::EnvConfig;

// Default behavior: struct name is used as prefix
#[derive(Debug, EnvConfig)]
struct DatabaseConfig {
    host: String, // -> DATABASE_CONFIG_HOST
    port: u16,    // -> DATABASE_CONFIG_PORT
    #[env_config(env = "DB_NAME")]
    database: String, // -> DB_NAME (field-level override)
}

// No prefix: use field names directly
#[derive(Debug, EnvConfig)]
#[env_config(no_prefix)]
struct ServerConfig {
    host: String, // -> HOST
    port: u16,    // -> PORT
}

// Custom prefix: use "APP" instead of struct name
#[derive(Debug, EnvConfig)]
#[env_config(prefix = "APP")]
struct ApplicationConfig {
    name: String,    // -> APP_NAME
    version: String, // -> APP_VERSION
}

fn main() -> Result<(), env_config::EnvConfigError> {
    // Set environment variables for demonstrations
    unsafe {
        // Default prefix (struct name)
        std::env::set_var("DATABASE_CONFIG_HOST", "db.example.com");
        std::env::set_var("DATABASE_CONFIG_PORT", "5432");
        std::env::set_var("DB_NAME", "production"); // Field-level override

        // No prefix
        std::env::set_var("HOST", "api.example.com");
        std::env::set_var("PORT", "8080");

        // Custom prefix
        std::env::set_var("APP_NAME", "MyApp");
        std::env::set_var("APP_VERSION", "1.0.0");
    }

    // Load configurations
    let db_config = DatabaseConfig::from_env()?;
    let server_config = ServerConfig::from_env()?;
    let app_config = ApplicationConfig::from_env()?;

    // Display results
    println!("1. Default behavior (struct name as prefix):");
    println!("{db_config:?}");

    println!("2. No prefix (#[env_config(no_prefix)]):");
    println!("{server_config:?}");

    println!("3. Custom prefix (#[env_config(prefix = \"APP\")]):");
    println!("{app_config:?}");

    Ok(())
}
