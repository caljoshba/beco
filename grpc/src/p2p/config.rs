use envconfig::Envconfig;

#[derive(Envconfig, Clone)]
pub struct Config {
    #[envconfig(from = "EXTERNAL_HOST")]
    pub external_host: String,

    #[envconfig(from = "P2P_PORT", default = "7000")]
    pub p2p_port: u32,

    #[envconfig(from = "EXTERNAL_P2P_PORT", default = "7000")]
    pub external_p2p_port: u32,

    #[envconfig(from = "RENDEZVOUS_ADDRESS")]
    pub rendezvous_address: String,
}