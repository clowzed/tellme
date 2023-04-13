#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub login: String,
    pub password: String,
    pub healthcheck_interval: u64,
}

impl Config {
    pub fn init() -> Config {
        let port     = std::env::var("PORT")
                                .expect("PORT environment variable must be set")
                                .parse()
                                .expect("Failed to parse PORT environment variable to u16");

        let login    = std::env::var("LOGIN").expect("LOGIN environment variable must be set");
        let password = std::env::var("PASSWORD").expect("PASSWORD environment variable must be set");

        let healthcheck_interval = std::env::var("HEALTHCHECK_INTERVAL")
                                            .expect("HEALTHCHECK_INTERVAL environment variable must be set")
                                            .parse()
                                            .expect("Failed to parse HEALTHCHECK_INTERVAL environment variable to u64");

        Config {
            port,
            login,
            password,
            healthcheck_interval,
        }
    }
}
