
use std::{net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};

use std::sync::Mutex;

use crate::{cluster::{Cluster, LBUtil}, config::ProxyConfig, conn::ConnectionHandler, http::endpoint::Endpoint, router::Router};

pub mod runtime;
pub mod listener;
pub mod config;
pub mod conn;
pub mod http;
pub mod cluster;
pub mod router;
pub mod pool;
fn main() -> anyhow::Result<()>{
    let config = std::env::var("PROXY_CFG").unwrap_or_else(|_| "config.yaml".to_string());
    let proxy_config = ProxyConfig::from_yml(PathBuf::from_str(config.as_ref()).unwrap()).expect("can not parse file");
    let rt = runtime::init();
    let lb_util = if proxy_config.init_lazy_pooling { 
        match proxy_config.clone().loadbalancer.as_str() { 
            "Round_Robin" => LBUtil::RoundRobin { current_index: Arc::new(Mutex::new(0 as usize)) },
            "Least_Connection" => LBUtil::LeastConnection,
            _ => LBUtil::RoundRobin { current_index: Arc::new(Mutex::new(0 as usize)) }
        }
    } else { 
        LBUtil::RoundRobin { current_index: Arc::new(Mutex::new(0 as usize)) }
    };
    let router = Router::new(
        proxy_config.clone().routes.into_iter().map(|route_c| {
            return (route_c.prefix, Arc::new(Cluster::new(
                route_c.cluster.name,
                route_c.cluster.endpoints.into_iter().map(|e_str| Endpoint::new(SocketAddr::from_str(&e_str).unwrap())).collect(),
                proxy_config.clone().init_lazy_pooling,
                lb_util.clone()
            )));
        }).collect()
    );
    let tcp_cluster = Cluster::new(
        "tcp_cluster",
        proxy_config.clone().tcp_hosts.into_iter().map(|s| Endpoint::new(SocketAddr::from_str(&s).unwrap())).collect(),
        proxy_config.clone().init_lazy_pooling,
        lb_util
    );
    let connection = ConnectionHandler::new(router, Arc::new(tcp_cluster));
    let proxy_config_clone = proxy_config.clone();
    rt.block_on(async { 
        let _ = listener::run( SocketAddr::from_str(&proxy_config.listener_addr).unwrap(), connection, proxy_config_clone).await;
    }); 
    anyhow::Ok(())
}
