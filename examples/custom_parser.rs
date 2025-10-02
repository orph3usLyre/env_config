use env_config::EnvConfig;

fn main() -> Result<(), env_config::EnvConfigError> {
    // Set some environment variables for demonstration
    //
    // # Safety
    // This example cannot run in parallel with other programs that set/remove ENV variables
    unsafe {
        std::env::set_var("PORT", "8080");
        std::env::set_var("ORIGIN", "0.0,0.0");
        std::env::set_var("APP_NAME", "my cool app");
        std::env::set_var("AUTHOR", "sappho");
        std::env::set_var("DESTINATION", "10.5,20.3");
    }

    let config = AppConfig::from_env()?;
    println!("Config: {:#?}", config);

    let expected = AppConfig {
        port: 8080,
        origin: Point { x: 0.0, y: 0.0 },
        application_name: "MY COOL APP".to_string(),
        author: Some("SAPPHO".to_string()),
        destination: Some(Point { x: 10.5, y: 20.3 }),
    };

    assert_eq!(config, expected);
    Ok(())
}

#[derive(Debug, EnvConfig, PartialEq, PartialOrd)]
#[env_config(no_prefix)]
struct AppConfig {
    // Regular field with automatic parsing
    port: u16, // -> PORT

    // Custom parser for complex type
    #[env_config(parse_with = "parse_point")]
    origin: Point, // -> ORIGIN

    // Custom parser with custom env var name
    #[env_config(env = "APP_NAME", parse_with = "to_uppercase")]
    application_name: String, // -> APP_NAME

    // Custom parser with optional custom env var name
    #[env_config(env = "AUTHOR", parse_with = "to_uppercase")]
    author: Option<String>, // -> AUTHOR

    // Optional field with custom parser
    #[env_config(parse_with = "parse_point")]
    destination: Option<Point>, // -> DESTINATION (automatically optional from Option<T> type)
}

// Custom parser that converts a string to uppercase
fn to_uppercase(s: String) -> String {
    s.to_uppercase()
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Point {
    x: f64,
    y: f64,
}

// Custom parser function that parses "x,y" format into Point
fn parse_point(s: String) -> Point {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        panic!("Point must be in format 'x,y'");
    }

    Point {
        x: parts[0].parse().expect("Invalid x coordinate"),
        y: parts[1].parse().expect("Invalid y coordinate"),
    }
}
