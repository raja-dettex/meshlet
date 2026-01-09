use std::net::SocketAddr;

use tokio::net::TcpStream;


pub struct Connection;


impl Connection {
    pub async fn handle(downstream: &mut TcpStream, upstream_addr: SocketAddr) -> anyhow::Result<()> { 
        println!("{:?}", upstream_addr);
        let mut upstream = TcpStream::connect(upstream_addr).await?;
        println!("{upstream:?}");
        let (_from_client, _from_upstream) = tokio::io::copy_bidirectional(downstream, &mut upstream).await?;
        anyhow::Ok(())
    }
}