use std::net::SocketAddr;

use tokio::net::TcpListener;
use crate::{config::Config, conn::Connection};



pub async fn run(listener_addr: SocketAddr, connection : Connection) -> anyhow::Result<()>{
    let listener = TcpListener::bind(listener_addr).await?;
    while let (mut stream, peer) = listener.accept().await? {
        if let Err(err) = connection.handle(&mut stream).await {
            println!("error handling connection for peer addr: {:?}, err: {:?}", peer, err);
        }
    };
    anyhow::Ok(())
}