use envconfig::Envconfig;

#[derive(Envconfig, Clone)]
pub struct Config {
    #[envconfig(from = "DB_HOST")]
    pub host: String,

    #[envconfig(from = "DB_PORT", default = "5432")]
    pub port: u16,

    #[envconfig(from = "DB_USER")]
    pub user: String,

    #[envconfig(from = "DB_PASSWORD")]
    pub password: String,

    #[envconfig(from = "DB_NAME")]
    pub database: String,

    #[envconfig(from = "DB_POOL_SIZE", default = "10")]
    pub pool_size: usize,
}