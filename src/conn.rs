use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream};
use tokio::time::{Duration, timeout};

use crate::http::server::{HttpServer};
use crate::router::Router;


const UPSTREAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
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
    if buf.starts_with(b"GET") {
        return anyhow::Ok(Protocol::Http1);
    }
    anyhow::Ok(Protocol::Unknown)
}

pub struct TcpHandle { 
    upstreams: Vec<SocketAddr>,
    index: Rc<RefCell<usize>>
}
impl TcpHandle { 
    pub fn new(upstreams: Vec<SocketAddr>) -> Self { 
        Self { upstreams, index: Rc::new(RefCell::new(0))}
    }

    pub fn select_upstream(&self) -> SocketAddr { 
        let mut i = self.index.borrow().to_owned();
        if i >= self.upstreams.len() { 
            i = 0;
        }
        let upstream = self.upstreams[i];
        *self.index.borrow_mut() = i + 1;
        upstream
    }
    pub async fn tcp_passthrough(&self, downstream: &mut TcpStream) -> anyhow::Result<()> {
            let upstream_addr = self.select_upstream();
            println!("{:?}", upstream_addr);
            let mut _state = ConnState::ConnectingUpstream;

            // upstream connect with timeout 
            let upstream = match timeout(UPSTREAM_CONNECT_TIMEOUT,
                TcpStream::connect(upstream_addr)).await {
                    Ok(Ok(s)) => s,
                    Ok(Err(e)) =>  { 
                        println!("connect failed with upstream for peer {} and error {}", upstream_addr, e); 
                        return anyhow::Ok(());
                    }
                    Err(_) => { 
                        println!("connection timeout with usptream peer {:?}", upstream_addr);
                        return anyhow::Ok(());
                    }
            };
            let mut upstream = upstream;
            _state = ConnState::Active;

            // bidirectional copy with idlke timeout
            
            println!("{upstream:?}");
            loop { 
                let copy = tokio::io::copy_bidirectional(downstream, &mut upstream);
                match timeout(IDLE_CONN_TIMEOUT, copy).await {
                    Ok(Ok((_from_client, _from_upstream))) => { 
                        // eof reached from one side
                        println!("end of copy with peer {:?}", upstream_addr);
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


pub async fn http_placeholder(_downstream: &TcpStream, upstream_addr: &SocketAddr) -> anyhow::Result<()> {
    println!("http connection detected with upstream peer : {:?}", upstream_addr);
    anyhow::Ok(())
}

pub struct Connection { 
    http_server: HttpServer,
    tcp_handle: TcpHandle
}


impl Connection {
    pub fn new(router: Router, tcp_upstreams: Vec<SocketAddr>) -> Self { 
        let http_server = HttpServer::new(Arc::new(router));
        let tcp_handle = TcpHandle::new(tcp_upstreams);
        Self { http_server, tcp_handle }
    }
    pub async fn handle(&self, mut downstream: &mut TcpStream) -> anyhow::Result<()> { 
        match detect_protocol(&downstream).await? {
            Protocol::Http1 => {
                self.http_server.serve_http(downstream).await
            },
            Protocol::Unknown => self.tcp_handle.tcp_passthrough(&mut downstream).await,
        }
    }
}