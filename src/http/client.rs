use std::net::SocketAddr;

use hyper::{Body, Client, Error, Request, Response};
use hyper::client::HttpConnector;

#[derive(Debug,Clone)]
pub struct HttpUpstream { 
    client: Client<HttpConnector>
}


impl HttpUpstream { 
    pub fn new() -> Self { 
        Self { 
            client: Client::new()
        }
    }


    pub async fn forward(&self, addr: SocketAddr, mut req: Request<Body>) -> Result<Response<Body>,Error> {
        let uri = format!("http://{}{}", addr, req.uri().path()); 
        *req.uri_mut() = uri.parse().unwrap();
        self.client.request(req).await
    }
}