use std::net::SocketAddr;

use tokio::net::TcpListener;
use crate::{config::ProxyConfig, conn::ConnectionHandler};



pub async fn run(listener_addr: SocketAddr, connection : ConnectionHandler, _config: ProxyConfig) -> anyhow::Result<()>{
    let listener = TcpListener::bind(listener_addr).await?;
    while let Ok((mut stream, peer)) = listener.accept().await {
        if let Err(err) = connection.handle(&mut stream).await {
            println!("error handling connection for peer addr: {:?}, err: {:?}", peer, err);
        }
    };
    anyhow::Ok(())
}