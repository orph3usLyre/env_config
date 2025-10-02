use env_config::{EnvConfig, EnvConfigError};

mod common;

// Test default behavior - should use struct name as prefix
#[derive(Debug, EnvConfig)]
struct DefaultPrefixConfig {
    database_url: String,
    port: u16,
}

// Test no_prefix attribute
#[derive(Debug, EnvConfig)]
#[env_config(no_prefix)]
struct NoPrefixConfig {
    database_url: String,
    port: u16,
}

// Test custom prefix attribute
#[derive(Debug, EnvConfig)]
#[env_config(prefix = "APP")]
struct CustomPrefixConfig {
    database_url: String,
    port: u16,
}

// Test mixed with field-level env attribute (should override prefix)
#[derive(Debug, EnvConfig)]
#[env_config(prefix = "TEST")]
struct MixedPrefixConfig {
    #[env_config(env = "CUSTOM_URL")]
    database_url: String,
    port: u16, // This should use TEST_PORT
}

#[test]
fn should_use_struct_name_as_default_prefix() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        (
            "DEFAULT_PREFIX_CONFIG_DATABASE_URL",
            "postgres://localhost/db",
        ),
        ("DEFAULT_PREFIX_CONFIG_PORT", "5432"),
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES, || DefaultPrefixConfig::from_env().unwrap())
    };

    assert_eq!(config.database_url, "postgres://localhost/db");
    assert_eq!(config.port, 5432);
}

#[test]
fn should_respect_no_prefix_attribute() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("DATABASE_URL", "postgres://localhost/db"),
        ("PORT", "5432"),
    ];
    let config =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, || NoPrefixConfig::from_env().unwrap()) };

    assert_eq!(config.database_url, "postgres://localhost/db");
    assert_eq!(config.port, 5432);
}

#[test]
fn should_use_custom_prefix() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("APP_DATABASE_URL", "postgres://localhost/db"),
        ("APP_PORT", "5432"),
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES, || CustomPrefixConfig::from_env().unwrap())
    };

    assert_eq!(config.database_url, "postgres://localhost/db");
    assert_eq!(config.port, 5432);
}

#[test]
fn should_allow_field_level_env_to_override_prefix() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("CUSTOM_URL", "postgres://localhost/db"), // Uses field-level env attribute
        ("TEST_PORT", "5432"),                     // Uses prefix from struct
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES, || MixedPrefixConfig::from_env().unwrap())
    };

    assert_eq!(config.database_url, "postgres://localhost/db");
    assert_eq!(config.port, 5432);
}

#[test]
fn should_fail_when_using_old_env_var_names_with_default_prefix() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("DATABASE_URL", "postgres://localhost/db"), // Old style without prefix
        ("PORT", "5432"),                            // Old style without prefix
    ];

    let result =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, DefaultPrefixConfig::from_env) };

    // Should fail because it's looking for DEFAULT_PREFIX_CONFIG_DATABASE_URL, not DATABASE_URL
    assert!(
        matches!(result, Err(EnvConfigError::Missing(var)) if var == "DEFAULT_PREFIX_CONFIG_DATABASE_URL")
    );
}
