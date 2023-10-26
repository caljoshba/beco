use envconfig::Envconfig;

#[derive(Envconfig, Clone)]
pub struct Config {
    #[envconfig(from = "EXTERNAL_HOST")]
    pub external_host: String,

    #[envconfig(from = "P2P_PORT", default = "7000")]
    pub p2p_port: u32,
}