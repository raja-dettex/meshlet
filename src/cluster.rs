use std::sync::Mutex;

use hyper::{Body, Error, Request, Response};

use crate::http::endpoint::Endpoint;

#[derive(Debug)]
pub struct Cluster { 
    pub name: String,
    endpoints: Vec<Endpoint>,
    rr_index: Mutex<usize>
}


impl Cluster { 
    pub fn new(name: impl Into<String>, endpoints: Vec<Endpoint>) -> Self { 
        Self { name: name.into(), endpoints, rr_index: Mutex::new(0)}
    }

    pub async fn forward(&self, req: Request<Body>) -> anyhow::Result<Response<Body>, Error> { 
        let endpoint = self.select_endpoint();
        endpoint.forward(req).await
    }

    pub fn select_endpoint(&self) -> Endpoint { 
        let mut index = self.rr_index.lock().unwrap().clone();
        if index < self.endpoints.len() { 
            *self.rr_index.lock().unwrap() = index + 1;
            return self.endpoints[index].clone();
        }
        index = 0;
        *self.rr_index.lock().unwrap() = index + 1;
        self.endpoints[index].clone()
    }
}