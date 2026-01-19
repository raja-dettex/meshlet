
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream};
use tokio::time::{Duration, timeout};

use crate::cluster::Cluster;
use crate::http::server::{HttpServer};
use crate::router::Router;


pub const UPSTREAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const IDLE_CONN_TIMEOUT : Duration = Duration::from_secs(30);
const HTTP_PEEK_LEN: usize = 8;
pub enum ConnState { 
    ConnectingUpstream,
    Active,
    Drained,
    TimedOut,
    Closed
}

#[derive(Debug)]
enum Protocol { 
    Http1,
    Unknown
}

async fn detect_protocol(downstream: &TcpStream) -> anyhow::Result<Protocol> {
    let mut buf = [0u8; HTTP_PEEK_LEN];
    let _ = downstream.peek(&mut buf).await?;
    if buf.starts_with(b"GET") || buf.starts_with(b"POST"){
        return anyhow::Ok(Protocol::Http1);
    }
    anyhow::Ok(Protocol::Unknown)
}

pub struct TcpHandle { 
    upstreams: Arc<Cluster>
}
impl TcpHandle { 
    pub fn new(upstreams: Arc<Cluster>) -> Self { 
        Self { upstreams}
    }

    pub async fn tcp_passthrough(&self, downstream: &mut TcpStream) -> anyhow::Result<()> {
            let endpoint = self.upstreams.select_endpoint();
            let upstream_addr = endpoint.addr();
            let (mut upstream, mut _state) = endpoint.connect().await?;
            

            // bidirectional copy with idlke timeout
            
            println!("{upstream:?}");
            loop { 
                let copy = tokio::io::copy_bidirectional(downstream, &mut upstream);
                match timeout(IDLE_CONN_TIMEOUT, copy).await {
                    Ok(Ok((_from_client, _from_upstream))) => { 
                        // eof reached from one side
                        println!("end of copy with remote peer: {}", upstream_addr);
                        _state = ConnState::Closed;
                        break;
                    },
                    Ok(Err(e)) => {
                        println!("copy error peer {:?} and error {}", upstream_addr, e);
                        _state = ConnState::Closed;
                        break;
                    }
                    Err(_) => {
                        println!("draining closing connection with peer {:?}", upstream_addr);
                        _state = ConnState::Drained;
                    },
                }
            }
            let _ = downstream.shutdown().await?;
            let _ = upstream.shutdown().await?;
            anyhow::Ok(())
    }
}




pub struct ConnectionHandler { 
    http_server: HttpServer,
    tcp_handle: TcpHandle
}


impl ConnectionHandler {
    pub fn new(router: Router, tcp_upstreams: Arc<Cluster>) -> Self { 
        let http_server = HttpServer::new(Arc::new(router));
        let tcp_handle = TcpHandle::new(tcp_upstreams);
        Self { http_server, tcp_handle }
    }
    pub async fn handle(&self, mut downstream: &mut TcpStream) -> anyhow::Result<()> { 
        match detect_protocol(&downstream).await? {
            Protocol::Http1 => {
                self.http_server.handle_http(downstream).await
            },
            Protocol::Unknown => self.tcp_handle.tcp_passthrough(&mut downstream).await,
        }
    }
}