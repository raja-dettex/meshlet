use std::net::SocketAddr;
pub struct Config { 
    pub listener_addr: SocketAddr,
    pub upstream_addr: SocketAddr
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> { 
        let listener_addr: SocketAddr = std::env::var("MESHLET_LISTENER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:10000".to_string()).parse()?;
        let upstream_addr: SocketAddr = std::env::var("MESHLET_UPSTREAM_ADDR")
        .unwrap_or_else(|_| "172.17.0.1:8080".to_string()).parse()?;
        anyhow::Ok(Self{
            listener_addr,
            upstream_addr
        })
    }
}