use std::net::SocketAddr;

use hyper::{Body, Client, Error, Request, Response, Uri};
use tokio::net::TcpStream;
use hyper::client::HttpConnector;

#[derive(Debug,Clone)]
pub struct HttpUpstream { 
    addr : SocketAddr,
    client: Client<HttpConnector>
}


impl HttpUpstream { 
    pub fn new(addr: SocketAddr) -> Self { 
        Self { 
            addr,
            client: Client::new()
        }
    }


    pub async fn forward(&self, mut req: Request<Body>) -> Result<Response<Body>,Error> {
        let uri = format!("http://{}{}", self.addr, req.uri().path()); 
        *req.uri_mut() = uri.parse().unwrap();
        self.client.request(req).await
    }
}