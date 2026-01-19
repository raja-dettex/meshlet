use std::sync::{Arc, Mutex};
use tokio::{net::TcpStream};

use hyper::{Body, Error, Request, Response};

use crate::{conn::ConnState, http::{client::HttpUpstream, endpoint::Endpoint}, pool::{ConnectionPool, PooledConn}};

#[derive(Debug, Clone)]
pub enum LBUtil { 
    RoundRobin{current_index: Arc<Mutex<usize>>},
    LeastConnection
}
#[derive(Debug)]
pub struct Cluster { 
    pub name: String,
    endpoints: Vec<Endpoint>,
    pool: Vec<Arc<ConnectionPool>>,
    rr_index: Mutex<usize>,
    http_upstream: HttpUpstream,
    lb_util: LBUtil
}

use futures::stream::{self,StreamExt};
impl Cluster { 
    pub fn new(name: impl Into<String>, endpoints: Vec<Endpoint>, init_lazy_pooling: bool, lb_util: LBUtil ) -> Self { 
        let pool = if init_lazy_pooling { 
            endpoints.clone().into_iter()
            .map(|e| ConnectionPool::new(e.addr(), 5)).collect()
        } else { 
            Vec::new()        
        };
        Self { 
            name: name.into(), 
            endpoints, 
            pool,
            rr_index: Mutex::new(0), 
            http_upstream: HttpUpstream::new(),
            lb_util
        }
    }


    pub async fn acquire_upstream_hosts(&self) -> std::io::Result<PooledConn> { 
        
        match &self.lb_util { 
            LBUtil::LeastConnection => { 
                let pool_clone = self.pool.clone();
                let pools: Vec<Arc<ConnectionPool>> = stream::iter(pool_clone).filter_map(async |pool| { 
                    if pool.is_pool_exhausted().await { 
                        return None;
                    }
                    Some(pool)
                }).collect().await;
                if let Some(pool) = pools.iter().next() { 
                    return Ok(pool.acquire().await?);
                }
                return Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "exhaused for both upstream hosts"))

            },
            LBUtil::RoundRobin { current_index } => { 
                let mut curr_index = current_index.lock().unwrap().to_owned();
            
                if curr_index >= self.pool.len() { 
                    curr_index = 0;
                }
                let pooled_conn = self.pool[curr_index].acquire().await?;
                *current_index.lock().unwrap() = curr_index + 1;
                Ok(pooled_conn)
            }
            
        }

    }

    pub async fn acquire_tcp(&self) -> std::io::Result<(TcpStream, ConnState)> {
        self.select_endpoint().connect().await
    }

    pub async fn forward(&self, req: Request<Body>) -> anyhow::Result<Response<Body>, Error> { 
        let endpoint = self.select_endpoint();
        self.http_upstream.forward(endpoint.addr(), req).await
    }

    

    pub fn select_endpoint(&self) -> Endpoint { 
        let mut index = self.rr_index.lock().unwrap().clone();
        if index < self.endpoints.len() { 
            *self.rr_index.lock().unwrap() = index + 1;
            return self.endpoints[index].clone();
        }
        index = 0;
        *self.rr_index.lock().unwrap() = index + 1;
        self.endpoints[index].clone()
    }
}