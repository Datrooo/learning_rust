pub struct Config {
    pub address: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            address: std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into()),
        }
    }
}
