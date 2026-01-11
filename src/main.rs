
use std::{net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};

use crate::{cluster::Cluster, config::ProxyConfig, conn::Connection, http::endpoint::Endpoint, router::Router};

pub mod runtime;
pub mod listener;
pub mod config;
pub mod conn;
pub mod http;
pub mod cluster;
pub mod router;
fn main() -> anyhow::Result<()>{
    let config = std::env::var("PROXY_CFG").unwrap_or_else(|_| "config.yaml".to_string());
    let proxy_config = ProxyConfig::from_yml(PathBuf::from_str(config.as_ref()).unwrap()).expect("can not parse file");
    println!("proxy config : {:#?}", proxy_config);
    let rt = runtime::init();
    
    let router = Router::new(
        proxy_config.routes.into_iter().map(|route_c| {
            return (route_c.prefix, Arc::new(Cluster::new(
                route_c.cluster.name,
                route_c.cluster.endpoints.into_iter().map(|e_str| Endpoint::new(SocketAddr::from_str(&e_str).unwrap())).collect()
            )));
        }).collect()
    );
    println!("router :: {router:#?}");
    let connection = Connection::new(router, proxy_config.tcp_hosts.into_iter().map(|s| SocketAddr::from_str(&s).unwrap()).collect());
    rt.block_on(async { 
        let _ = listener::run(SocketAddr::from_str(&proxy_config.listener_addr).unwrap(), connection).await;
    }); 
    anyhow::Ok(())
}
