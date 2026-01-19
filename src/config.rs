use std::{fs::File, io::{Read}, net::SocketAddr, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ProxyConfig { 
    pub name: String,
    pub listener_addr: String,
    pub init_lazy_pooling: bool,
    pub loadbalancer: String,
    pub routes: Vec<RouterConfig>,
    pub tcp_hosts: Vec<String>
}

#[derive(Debug, Deserialize, Clone)]
pub struct RouterConfig { 
    pub prefix: String, 
    pub cluster: ClusterConfig
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClusterConfig { 
    pub name: String,
    pub endpoints: Vec<String>
}


impl ProxyConfig { 
    pub fn from_yml(path: PathBuf) -> std::io::Result<Self> { 
        let mut file = File::open(path)?;
        let mut buf = String::new();
        let n = file.read_to_string(&mut buf)?;
        let contents = &buf[..n];
        let proxy_config : ProxyConfig = serde_yml::from_str(contents).
            map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string()))?;
        Ok(proxy_config)
    }
}
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