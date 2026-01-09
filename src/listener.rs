use tokio::net::TcpListener;
use crate::{config::Config, conn::Connection};



pub async fn run(config: Config) -> anyhow::Result<()>{
    let listener = TcpListener::bind(config.listener_addr).await?;
    while let (mut stream, peer) = listener.accept().await? {
        if let Err(err) = Connection::handle(&mut stream, config.upstream_addr).await {
            println!("error handling connection for peer addr: {:?}, err: {:?}", peer, err);
        }
    };
    anyhow::Ok(())
}