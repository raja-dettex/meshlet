use std::sync::Arc;

use hyper::{Body, Request};

use crate::cluster::Cluster;

#[derive(Debug)]
pub struct Router { 
    routes: Vec<Route>
}

#[derive(Debug)]
pub struct Route { 
    prefix: String,
    cluster: Arc<Cluster> 
}


impl Router {
    pub fn new(routes: Vec<(String, Arc<Cluster>)>) -> Self { 
        let routes : Vec<Route> = routes.into_iter().
        map(|(prefix, cluster)| Route{ prefix, cluster}).collect();
        Self { routes}
    }

    pub fn route(&self, req: &Request<Body>) -> Arc<Cluster>{
        let path = req.uri().path();
        for route in &self.routes { 
            if path.starts_with(&route.prefix) { 
                return route.cluster.clone();
            }
        }
        // fallback to default cluster
        self.routes[0].cluster.clone()
    }
}