use std::net::SocketAddr;
use std::sync::LazyLock;

use axum_client_ip::ClientIpSource;
use envconfig::Envconfig;

pub static API_CONFIG: LazyLock<ApiConfig> = LazyLock::new(|| ApiConfig::init_from_env().unwrap());

#[derive(Envconfig)]
pub struct ApiConfig {
    #[envconfig(from = "API_ADDRESS", default = "127.0.0.1:8005")]
    pub address: SocketAddr,
    #[envconfig(from = "API_CLIENT_IP_SOURCE", default = "ConnectInfo")]
    pub client_ip_source: ClientIpSource,
}
