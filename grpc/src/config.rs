use envconfig::Envconfig;

#[derive(Envconfig, Clone)]
pub struct Config {
    #[envconfig(from = "GRPC_PORT", default = "9000")]
    pub grpc_port: u16,

    #[envconfig(from = "EXTERNAL_GRPC_PORT", default = "9000")]
    pub external_grpc_port: u16,
}