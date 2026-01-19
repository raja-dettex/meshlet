use std::{io::ErrorKind, net::SocketAddr};
use tokio::{net::TcpStream, time::timeout};

use crate::conn::{ConnState, UPSTREAM_CONNECT_TIMEOUT};

#[derive(Debug, Clone)]
pub struct Endpoint { 
    addr: SocketAddr
}

impl Endpoint { 
    pub fn new(addr: SocketAddr) -> Self { 
        Self { 
            addr
        }
    }

    pub async fn connect(&self) -> std::io::Result<(TcpStream,ConnState)> { 
        let mut _state = ConnState::ConnectingUpstream;
            let upstream_addr = self.addr;
            // upstream connect with timeout 
            let upstream = match timeout(UPSTREAM_CONNECT_TIMEOUT,
                TcpStream::connect(upstream_addr)).await {
                    Ok(Ok(s)) => s,
                    Ok(Err(e)) =>  { 
                        println!("connect failed with upstream for peer {} and error {}", upstream_addr, e); 
                        return Err(e);
                    }
                    Err(e) => { 
                        println!("connection timeout with usptream peer {:?}", upstream_addr);
                        return Err(std::io::Error::new(ErrorKind::TimedOut, e.to_string()));
                    }
            };
            _state = ConnState::Active;
            Ok((upstream, _state))
    }

    pub fn addr(&self) -> SocketAddr { 
        self.addr
    }
}