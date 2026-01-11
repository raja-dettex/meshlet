use std::net::SocketAddr;

use hyper::{Body, Error, Request, Response};

use crate::http::client::HttpUpstream;

#[derive(Debug, Clone)]
pub struct Endpoint { 
    upstream : HttpUpstream
}

impl Endpoint { 
    pub fn new(addr: SocketAddr) -> Self { 
        let upstream = HttpUpstream::new(addr);
        Self { 
            upstream
        }
    }

    pub async fn forward(&self, req: Request<Body>) -> Result<Response<Body>, Error > { 
        self.upstream.forward(req).await
    }
}