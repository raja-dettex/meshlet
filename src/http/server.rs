
use std::sync::Arc;

use hyper::{Error, Request, Response, body::Body, server::conn::Http, service::service_fn};
use tokio::net::TcpStream;

use crate::router::Router;

pub struct HttpServer { 
    router: Arc<Router>
}

impl HttpServer { 
    pub fn new(router: Arc<Router>) -> Self{ 
        Self { router}
    }
    pub async fn serve_http(
        &self,
        stream: &mut TcpStream
    ) -> anyhow::Result<()> {
        let service_fn = service_fn(move |req| {
            let router = self.router.clone();
            async move {
                Self::handle_request(req, router).await
            }
        });
        Http::new().http1_keep_alive(true).serve_connection(stream, service_fn).await?;
        anyhow::Ok(())
    }


    pub async fn handle_request(req: Request<Body>, router: Arc<Router>) -> Result<Response<Body>, Error> {
        let cluster = router.route(&req);
        cluster.forward(req).await
    }
}