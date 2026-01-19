use bytes::BytesMut;
use httparse::{Request, Response};
use tokio::{io::AsyncReadExt, net::TcpStream};

pub struct HttpRequest { 
    pub path: String,
    pub raw_headers: BytesMut,
    pub body: BytesMut
}

impl HttpRequest { 
   pub async fn parse(stream: &mut TcpStream) -> anyhow::Result<Self> {
    let mut buffer = BytesMut::with_capacity(4096);

    loop {
        let n = stream.read_buf(&mut buffer).await?;
        if n == 0 {
            anyhow::bail!("downstream closed");
        }

        let mut headers = [httparse::EMPTY_HEADER; 32];
        let mut req = Request::new(&mut headers);
        let buf_clone = buffer.clone();
        let hdr_len = match req.parse(&buf_clone)? {
            httparse::Status::Complete(len) => len,
            httparse::Status::Partial => continue,
        };

        let path = req.path.ok_or_else(|| anyhow::anyhow!("no path"))?;

        let mut content_len = 0usize;
        for h in req.headers {
            if h.name.eq_ignore_ascii_case("content-length") {
                content_len = std::str::from_utf8(h.value)?.parse()?;
            }
        }

        // split headers cleanly
        let raw_headers = buffer.split_to(hdr_len);

        // buffer now holds body (maybe partial)
        while buffer.len() < content_len {
            let n = stream.read_buf(&mut buffer).await?;
            if n == 0 {
                anyhow::bail!("EOF while reading request body");
            }
        }

        let body = buffer.split_to(content_len);

        return Ok(Self {
            path: path.to_string(),
            raw_headers,
            body,
        });
    }
}

}


#[derive(Debug)]
pub struct HttpResponse { 
    pub raw_headers: BytesMut,
    pub body: BytesMut,
    pub keep_alive: bool
}

impl HttpResponse { 
    pub async fn parse(stream: &mut TcpStream) -> anyhow::Result<Self> {
        let mut buffer = BytesMut::with_capacity(8192);

        loop {
            let n = stream.read_buf(&mut buffer).await?;
            if n == 0 {
                anyhow::bail!("upstream closed");
            }

            let mut headers = [httparse::EMPTY_HEADER; 32];
            let mut resp = Response::new(&mut headers);

            let hdr_len = match resp.parse(&buffer)? {
                httparse::Status::Complete(len) => len,
                httparse::Status::Partial => continue,
            };

            let mut keep_alive = true;
            let mut content_len = 0usize;

            for h in resp.headers {
                if h.name.eq_ignore_ascii_case("connection")
                    && h.value.eq_ignore_ascii_case(b"close")
                {
                    keep_alive = false;
                }
                if h.name.eq_ignore_ascii_case("content-length") {
                    content_len = std::str::from_utf8(h.value)?.parse()?;
                }
            }

            let raw_headers = buffer.split_to(hdr_len);

            while buffer.len() < content_len {
                let n = stream.read_buf(&mut buffer).await?;
                if n == 0 {
                    anyhow::bail!("EOF during body read");
                }
            }

            let body = buffer.split_to(content_len);

            return Ok(Self {
                raw_headers,
                body,
                keep_alive,
            });
        }
    }

}