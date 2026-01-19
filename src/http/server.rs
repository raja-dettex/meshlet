
use std::sync::Arc;


use hyper::{Error, Request, Response, body::Body, server::conn::Http, service::service_fn};
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{http::parser::{HttpRequest, HttpResponse}, router::Router};

pub struct HttpServer { 
    router: Arc<Router>
}

impl HttpServer { 
    pub fn new(router: Arc<Router>) -> Self{ 
        Self { router}
    }

    pub async fn handle_http(
        &self,
        stream: &mut TcpStream
    ) -> anyhow::Result<()> { 
        println!("handling connection");
        let request = HttpRequest::parse(stream).await?;
        let cluster = self.router.route_http(&request);
        let mut pooled_conn = cluster.acquire_upstream_hosts().await?;
        let upstream = pooled_conn.stream();
        let body_str = std::str::from_utf8(&request.body.as_ref())?;
        println!("request body {:?}", body_str);
        let _ = upstream.write_all(&request.raw_headers).await?;
        let _ = upstream.write_all(&request .body).await?;
        let response = HttpResponse::parse(upstream).await?;
        println!("received response : {:?}", response);
        // write the response to downstream
        let _ = stream.write_all(&response.raw_headers).await?;
        let _ = stream.write_all(&response.body).await?;
        anyhow::Ok(())
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