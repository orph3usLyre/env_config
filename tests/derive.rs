// Derive macro tests
use env_config::{EnvConfig, EnvConfigError};

mod common;

#[derive(Debug, EnvConfig)]
#[env_config(no_prefix)]
struct AutoDerivedConfig {
    database_url: String, // -> DATABASE_URL
    api_key: String,      // -> API_KEY
    max_connections: u32, // -> MAX_CONNECTIONS
    #[env_config(default = "8080")]
    port: u16, // -> PORT (with default)
    timeout: Option<u64>, // -> TIMEOUT (optional)
    #[env_config(skip)]
    internal_state: String, // Skipped - not loaded from env, uses Default::default()
}

#[derive(Debug, EnvConfig)]
struct CustomAttributesDerivedConfig {
    #[env_config(env = "DB_HOST")]
    host: String, // -> DB_HOST (custom name)
    #[env_config(env = "DB_PORT", default = "5432")]
    port: u16, // -> DB_PORT (custom name + default)
    #[env_config(env = "DB_TIMEOUT")]
    timeout: Option<u64>, // -> DB_TIMEOUT (custom name + optional)
    #[env_config(skip)]
    connection_pool: Vec<String>, // Skipped, uses Default::default()
}

#[derive(Debug, EnvConfig)]
#[env_config(no_prefix)]
struct TypeVarietyTest {
    string_field: String,            // -> STRING_FIELD
    int_field: i32,                  // -> INT_FIELD
    float_field: f64,                // -> FLOAT_FIELD
    bool_field: bool,                // -> BOOL_FIELD
    optional_int: Option<i32>,       // -> OPTIONAL_INT
    optional_string: Option<String>, // -> OPTIONAL_STRING
}

#[derive(Debug, EnvConfig)]
#[env_config(no_prefix)]
struct MixedAttributesTest {
    #[env_config(env = "CUSTOM_NAME")]
    field1: String, // -> CUSTOM_NAME
    #[env_config(env = "ANOTHER_CUSTOM", default = "42")]
    field2: i32, // -> ANOTHER_CUSTOM (with default)
    #[env_config(env = "OPTIONAL_CUSTOM")]
    field3: Option<String>, // -> OPTIONAL_CUSTOM (optional)
    auto_field: String, // -> AUTO_FIELD (auto snake_case)
    #[env_config(skip)]
    skipped_field: Vec<i32>, // Skipped
}

#[derive(Debug, EnvConfig)]
#[env_config(no_prefix)]
struct CustomParserTest {
    #[env_config(parse_with = "parse_custom_struct")]
    custom_field: CustomStruct, // -> CUSTOM_FIELD
    #[env_config(env = "DOUBLED", parse_with = "parse_doubled_int")]
    doubled_value: i32, // -> DOUBLED (custom name + custom parser)
    #[env_config(parse_with = "parse_doubled_int")]
    optional_doubled: Option<i32>, // -> OPTIONAL_DOUBLED (optional + custom parser)
    normal_field: String, // -> NORMAL_FIELD
}

// Test custom parser functionality
#[derive(Debug, Clone, PartialEq)]
struct CustomStruct {
    value: i32,
    name: String,
}

fn parse_custom_struct(s: String) -> CustomStruct {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        panic!("Invalid format for CustomStruct");
    }
    CustomStruct {
        value: parts[0].parse().expect("Invalid number"),
        name: parts[1].to_string(),
    }
}

fn parse_doubled_int(s: String) -> i32 {
    let base: i32 = s.parse().expect("Invalid number");
    base * 2
}

#[test]
fn should_parse_auto_derived_config_fields() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("DATABASE_URL", "postgres://localhost/db"),
        ("API_KEY", "secret123"),
        ("MAX_CONNECTIONS", "100"),
        ("PORT", "3000"),
        ("TIMEOUT", "60"),
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES, || AutoDerivedConfig::from_env().unwrap())
    };

    assert_eq!(config.database_url, "postgres://localhost/db");
    assert_eq!(config.api_key, "secret123");
    assert_eq!(config.max_connections, 100);
    assert_eq!(config.port, 3000);
    assert_eq!(config.timeout, Some(60));
    assert_eq!(config.internal_state, ""); // Default value since it's skipped
}

#[test]
fn should_parse_with_defaults_and_skip() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("DATABASE_URL", "postgres://localhost/db"),
        ("API_KEY", "secret123"),
        ("MAX_CONNECTIONS", "100"),
        // Don't set PORT, TIMEOUT to test defaults/optional
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES, || AutoDerivedConfig::from_env().unwrap())
    };

    assert_eq!(config.database_url, "postgres://localhost/db");
    assert_eq!(config.api_key, "secret123");
    assert_eq!(config.max_connections, 100);
    assert_eq!(config.port, 8080); // default
    assert_eq!(config.timeout, None); // optional, not set
    assert_eq!(config.internal_state, ""); // skipped, gets default
}

#[test]
fn should_parse_custom_names() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("DB_HOST", "localhost"),
        ("DB_PORT", "5433"),
        ("DB_TIMEOUT", "30"),
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES, || {
            CustomAttributesDerivedConfig::from_env().unwrap()
        })
    };

    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 5433);
    assert_eq!(config.timeout, Some(30));
    assert_eq!(config.connection_pool, Vec::<String>::new()); // skipped, gets default
}

#[test]
fn should_parse_custom_names_with_defaults() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("DB_HOST", "localhost"),
        // Don't set DB_PORT and DB_TIMEOUT to test defaults
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES, || {
            CustomAttributesDerivedConfig::from_env().unwrap()
        })
    };

    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 5432); // default
    assert_eq!(config.timeout, None); // optional, not set
    assert_eq!(config.connection_pool, Vec::<String>::new()); // skipped, gets default
}

#[test]
fn should_handle_case_conversion() {
    #[derive(Debug, EnvConfig)]
    #[env_config(no_prefix)]
    struct SnakeCaseTest {
        simple_field: String,          // -> SIMPLE_FIELD
        very_long_field_name: String,  // -> VERY_LONG_FIELD_NAME
        field_with_numbers123: String, // -> FIELD_WITH_NUMBERS123
        single: String,                // -> SINGLE
    }

    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("SIMPLE_FIELD", "simple"),
        ("VERY_LONG_FIELD_NAME", "very_long"),
        ("FIELD_WITH_NUMBERS123", "with_numbers"),
        ("SINGLE", "single"),
    ];
    let config =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, || SnakeCaseTest::from_env().unwrap()) };
    assert_eq!(config.simple_field, "simple");
    assert_eq!(config.very_long_field_name, "very_long");
    assert_eq!(config.field_with_numbers123, "with_numbers");
    assert_eq!(config.single, "single");
}

#[test]
fn should_parse_type_variety() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("STRING_FIELD", "test_string"),
        ("INT_FIELD", "42"),
        ("FLOAT_FIELD", "3.14"),
        ("BOOL_FIELD", "true"),
        ("OPTIONAL_INT", "123"),
        ("OPTIONAL_STRING", "optional_value"),
    ];
    let config =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, || TypeVarietyTest::from_env().unwrap()) };

    assert_eq!(config.string_field, "test_string");
    assert_eq!(config.int_field, 42);
    assert_eq!((config.float_field - 3.14).abs() < f64::EPSILON, true);
    assert_eq!(config.bool_field, true);
    assert_eq!(config.optional_int, Some(123));
    assert_eq!(config.optional_string, Some("optional_value".to_string()));
}

#[test]
fn should_parse_type_variety_with_missing_optionals() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("STRING_FIELD", "test_string"),
        ("INT_FIELD", "42"),
        ("FLOAT_FIELD", "3.14"),
        ("BOOL_FIELD", "false"),
        // Don't set optional fields
    ];
    let config =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, || TypeVarietyTest::from_env().unwrap()) };
    assert_eq!(config.string_field, "test_string");
    assert_eq!(config.int_field, 42);
    assert_eq!((config.float_field - 3.14).abs() < f64::EPSILON, true);
    assert_eq!(config.bool_field, false);
    assert_eq!(config.optional_int, None);
    assert_eq!(config.optional_string, None);
}

#[test]
fn should_err_with_missing_required_field() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("WITH_DEFAULT", "custom_value"),
        // Don't set optional fields
    ];

    #[derive(Debug, EnvConfig)]
    #[env_config(no_prefix)]
    #[allow(dead_code)]
    struct ErrorHandlingTest {
        #[env_config(default = "default_value")]
        with_default: String, // -> WITH_DEFAULT
        optional_field: Option<String>, // -> OPTIONAL_FIELD
        required_field: String,         // -> REQUIRED_FIELD
    }

    let result =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, || ErrorHandlingTest::from_env()) };
    assert!(matches!(result, Err(EnvConfigError::Missing(var)) if var == "REQUIRED_FIELD"));
}

#[test]
fn should_err_if_fields_cannot_be_parsed() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("STRING_FIELD", "valid_string"),
        ("INT_FIELD", "not_a_number"),
        ("FLOAT_FIELD", "3.14"),
        ("BOOL_FIELD", "true"),
        // Don't set optional fields
    ];
    let result = unsafe { common::with_env_vars(ENV_KEYS_VALUES, || TypeVarietyTest::from_env()) };

    assert!(matches!(result, Err(EnvConfigError::Parse(var, _)) if var == "INT_FIELD"));
}

#[test]
fn should_parse_mixed_attributes() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("CUSTOM_NAME", "custom_value"),
        ("ANOTHER_CUSTOM", "999"),
        ("OPTIONAL_CUSTOM", "optional_value"),
        ("AUTO_FIELD", "auto_value"),
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES, || MixedAttributesTest::from_env().unwrap())
    };

    assert_eq!(config.field1, "custom_value");
    assert_eq!(config.field2, 999);
    assert_eq!(config.field3, Some("optional_value".to_string()));
    assert_eq!(config.auto_field, "auto_value");
    assert_eq!(config.skipped_field, Vec::<i32>::new()); // Default
}

#[test]
fn should_parse_mixed_attributes_with_defaults() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("CUSTOM_NAME", "custom_value"),
        ("AUTO_FIELD", "auto_value"),
        // Don't set ANOTHER_CUSTOM or OPTIONAL_CUSTOM to test defaults
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES, || MixedAttributesTest::from_env().unwrap())
    };
    assert_eq!(config.field1, "custom_value");
    assert_eq!(config.field2, 42); // Default value
    assert_eq!(config.field3, None); // Optional, not set
    assert_eq!(config.auto_field, "auto_value");
    assert_eq!(config.skipped_field, Vec::<i32>::new()); // Default
}

#[test]
fn should_parse_with_custom_parser() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("CUSTOM_FIELD", "42,test_name"),
        ("DOUBLED", "10"),
        ("OPTIONAL_DOUBLED", "5"),
        ("NORMAL_FIELD", "normal_value"),
    ];
    let config =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, || CustomParserTest::from_env().unwrap()) };

    assert_eq!(
        config.custom_field,
        CustomStruct {
            value: 42,
            name: "test_name".to_string()
        }
    );
    assert_eq!(config.doubled_value, 20); // 10 * 2
    assert_eq!(config.optional_doubled, Some(10)); // 5 * 2
    assert_eq!(config.normal_field, "normal_value");
}

#[test]
fn should_parse_with_custom_parser_and_missing_optional() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("CUSTOM_FIELD", "99,another_name"),
        ("DOUBLED", "7"),
        ("NORMAL_FIELD", "another_normal_value"),
        // Don't set OPTIONAL_DOUBLED
    ];
    let config =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, || CustomParserTest::from_env().unwrap()) };

    assert_eq!(
        config.custom_field,
        CustomStruct {
            value: 99,
            name: "another_name".to_string()
        }
    );
    assert_eq!(config.doubled_value, 14); // 7 * 2
    assert_eq!(config.optional_doubled, None); // Not set
    assert_eq!(config.normal_field, "another_normal_value");
}

#[test]
fn should_parse_with_all_fields_skipped() {
    #[derive(Debug, EnvConfig)]
    #[env_config(no_prefix)]
    struct AllSkippedTest {
        #[env_config(skip)]
        field1: String,
        #[env_config(skip)]
        field2: i32,
        #[env_config(skip)]
        field3: Vec<String>,
    }

    // Don't set any environment variables
    //
    // # Safety
    // This is an exception that does not need to be run with `common::with_env_vars`
    // because it does not read any ENV variables
    let config = AllSkippedTest::from_env().unwrap();
    assert_eq!(config.field1, String::default());
    assert_eq!(config.field2, i32::default());
    assert_eq!(config.field3, Vec::<String>::default());
}

#[test]
fn should_parse_boolean_variants() {
    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("STRING_FIELD", "test"),
        ("INT_FIELD", "1"),
        ("FLOAT_FIELD", "1.0"),
        ("BOOL_FIELD", "true"),
    ];
    let config =
        unsafe { common::with_env_vars(ENV_KEYS_VALUES, || TypeVarietyTest::from_env().unwrap()) };
    assert_eq!(config.bool_field, true);

    const ENV_KEYS_VALUES_2: &[(&str, &str)] = &[
        ("STRING_FIELD", "test"),
        ("INT_FIELD", "1"),
        ("FLOAT_FIELD", "1.0"),
        ("BOOL_FIELD", "false"),
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES_2, || TypeVarietyTest::from_env().unwrap())
    };

    assert_eq!(config.bool_field, false);
}

#[test]
fn should_parse_with_complex_defaults() {
    #[derive(Debug, EnvConfig)]
    #[env_config(no_prefix)]
    struct ComplexDefaultsTest {
        #[env_config(default = "localhost")]
        host: String,
        #[env_config(default = "5432")]
        port: u16,
        #[env_config(default = "false")]
        ssl: bool,
        #[env_config(default = "30")]
        timeout: u64,
        #[env_config(default = "3.14")]
        rate: f64,
    }

    // Test without any env vars set
    //
    // # Safety
    // This is an exception that does not need to be run with `common::with_env_vars`
    // because it does not read any ENV variables
    let config = ComplexDefaultsTest::from_env().unwrap();
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 5432);
    assert_eq!(config.ssl, false);
    assert_eq!(config.timeout, 30);
    assert_eq!((config.rate - 3.14).abs() < f64::EPSILON, true);
}

#[test]
fn should_parse_edge_case_field_names() {
    #[derive(Debug, EnvConfig)]
    #[env_config(no_prefix)]
    struct EdgeCaseNamesTest {
        a: String,      // -> A
        ab: String,     // -> AB
        a_b: String,    // -> A_B
        a_b_c: String,  // -> A_B_C
        field_: String, // -> FIELD_
        _field: String, // -> _FIELD
    }

    const ENV_KEYS_VALUES: &[(&str, &str)] = &[
        ("A", "value_a"),
        ("AB", "value_ab"),
        ("A_B", "value_a_b"),
        ("A_B_C", "value_a_b_c"),
        ("FIELD_", "value_field_"),
        ("_FIELD", "value__field"),
    ];
    let config = unsafe {
        common::with_env_vars(ENV_KEYS_VALUES, || EdgeCaseNamesTest::from_env().unwrap())
    };

    assert_eq!(config.a, "value_a");
    assert_eq!(config.ab, "value_ab");
    assert_eq!(config.a_b, "value_a_b");
    assert_eq!(config.a_b_c, "value_a_b_c");
    assert_eq!(config.field_, "value_field_");
    assert_eq!(config._field, "value__field");
}
