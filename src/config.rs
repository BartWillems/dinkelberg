#[derive(Deserialize, Debug)]
pub struct Config {
    bot_name: String,
    redis_url: Option<String>,
    opentelemetry_endpoint: Option<String>,
}

lazy_static! {
    static ref CONFIG: Config = match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("Missing or incorrect environment variable: {}", error),
    };
}

impl Config {
    pub fn bot_name() -> &'static str {
        &CONFIG.bot_name
    }

    pub fn redis_url() -> Option<&'static str> {
        CONFIG.redis_url.as_ref().map(|url| url.as_ref())
    }

    pub fn opentelemetry_endpoint() -> &'static str {
        match &CONFIG.opentelemetry_endpoint {
            Some(endpoint) => endpoint.as_ref(),
            None => "127.0.0.1:6831",
        }
    }
}
