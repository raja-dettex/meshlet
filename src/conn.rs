use std::net::SocketAddr;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout, Instant};


const UPSTREAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const IDLE_CONN_TIMEOUT : Duration = Duration::from_secs(30);

pub enum ConnState { 
    ConnectingUpstream,
    Active,
    Drained,
    TimedOut,
    Closed
}

pub struct Connection;


impl Connection {
    pub async fn handle(downstream: &mut TcpStream, upstream_addr: SocketAddr) -> anyhow::Result<()> { 
        println!("{:?}", upstream_addr);
        let mut state = ConnState::ConnectingUpstream;

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
        state = ConnState::Active;

        // bidirectional copy with idlke timeout
        
        println!("{upstream:?}");
        loop { 
            let copy = tokio::io::copy_bidirectional(downstream, &mut upstream);
            match timeout(IDLE_CONN_TIMEOUT, copy).await {
                Ok(Ok((_from_client, _from_upstream))) => { 
                    // eof reached from one side
                    println!("end of copy with peer {:?}", upstream_addr);
                    state = ConnState::Closed;
                    break;
                },
                Ok(Err(e)) => {
                    println!("copy error peer {:?} and error {}", upstream_addr, e);
                    state = ConnState::Closed;
                    break;
                }
                Err(_) => {
                    println!("draining closing connection with peer {:?}", upstream_addr);
                    state = ConnState::Drained;
                },
            }
        }
        let _ = downstream.shutdown().await?;
        let _ = upstream.shutdown().await?;
        anyhow::Ok(())
    }
}