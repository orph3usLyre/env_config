// Manual implementation tests (what users can do without macros)
use env_config::{EnvConfig, EnvConfigError, env_var, env_var_optional};

mod common;

#[derive(Debug)]
struct ManualConfig {
    nats_auth: String,
    nats_seed: String,
    port: u16,
    timeout: Option<u64>,
    debug: bool,
}

impl EnvConfig for ManualConfig {
    type Error = EnvConfigError;

    fn from_env() -> Result<Self, Self::Error> {
        Ok(ManualConfig {
            nats_auth: env_var("NATS_AUTH")?,
            nats_seed: env_var("NATS_SEED")?,
            port: env_var_optional("PORT")?.unwrap_or(8080),
            timeout: env_var_optional("TIMEOUT")?,
            debug: env_var_optional("DEBUG")?.unwrap_or(false),
        })
    }
}

#[test]
fn should_parse_from_manual_impl() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("NATS_AUTH", "test_auth"),
        ("NATS_SEED", "test_seed"),
        ("PORT", "9090"),
        ("TIMEOUT", "30"),
        ("DEBUG", "true"),
    ];

    let config =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, || ManualConfig::from_env().unwrap()) };

    assert_eq!(config.nats_auth, "test_auth");
    assert_eq!(config.nats_seed, "test_seed");
    assert_eq!(config.port, 9090);
    assert_eq!(config.timeout, Some(30));
    assert_eq!(config.debug, true);
}

#[test]
fn should_parse_manual_impl_with_defaults() {
    const ENV_KEYS_VALUES: &[(&str, &str)] =
        &[("NATS_AUTH", "test_auth"), ("NATS_SEED", "test_seed")];

    let config =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, || ManualConfig::from_env().unwrap()) };

    assert_eq!(config.nats_auth, "test_auth");
    assert_eq!(config.nats_seed, "test_seed");
    assert_eq!(config.port, 8080); // default
    assert_eq!(config.timeout, None); // optional, not set
    assert_eq!(config.debug, false); // default
}

#[test]
fn should_err_if_missing_required_var() {
    let result = unsafe { common::with_env_vars(&[], || ManualConfig::from_env()) };
    dbg!(&result);
    assert!(matches!(result, Err(EnvConfigError::Missing(_))));
}

#[test]
fn should_err_if_field_is_not_parseable() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("NATS_AUTH", "test_auth"),
        ("NATS_SEED", "test_seed"),
        ("PORT", "not_a_number"),
    ];

    let result = unsafe { common::with_env_vars(ENV_KEYS_VALUES, || ManualConfig::from_env()) };
    assert!(matches!(result, Err(EnvConfigError::Parse(_, _))));
}
