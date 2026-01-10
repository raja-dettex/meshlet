
use hyper::{Error, Request, Response, body::Body, server::conn::Http, service::service_fn};
use tokio::net::TcpStream;

use crate::http::endpoint::Upstream;

pub async fn service_http(
    stream: &mut TcpStream,
    upstream: Upstream
) -> anyhow::Result<()> {
    let service_fn = service_fn(move |req| {
        let upstream = upstream.clone();
        async move {
            handle_request(req, upstream).await
        }
    });
    Http::new().http1_keep_alive(true).serve_connection(stream, service_fn).await?;
    anyhow::Ok(())
}


pub async fn handle_request(req: Request<Body>, upstream: Upstream) -> Result<Response<Body>, Error> {
    upstream.forward(req).await
}