use tokio::sync::Mutex;
use std::{io::ErrorKind, sync::Arc};

use tokio::net::TcpStream;

#[derive(Debug)]
pub struct ConnectionPool { 
    addr: std::net::SocketAddr,
    inner: Mutex<PoolInner>,
    max_conns: usize       
}

#[derive(Debug)]
pub struct PoolInner { 
    idle: Vec<TcpStream>,
    active: usize
}

pub struct PooledConn { 
    pub stream: Option<TcpStream>,
    pool: Arc<ConnectionPool>
}

impl ConnectionPool { 
    pub fn new(addr: std::net::SocketAddr, max_conns: usize) -> Arc<Self> { 
        Arc::new(Self { 
            addr,
            inner: Mutex::new(PoolInner { idle: Vec::new(), active: 0 }),
            max_conns
        })
    }

    pub async fn acquire(self: &Arc<Self>) -> std::io::Result<PooledConn> { 
        let mut inner = self.inner.lock().await;
        println!("idle connections are {:?}", inner.idle.len());
        if let Some(stream) = inner.idle.pop() { 
            return Ok(PooledConn { 
                stream: Some(stream),
                pool: Arc::clone(self)
            });
        }
        if inner.active < self.max_conns { 
            let stream = TcpStream::connect(self.addr).await?;
            inner.active += 1;
            drop(inner);
            return Ok(PooledConn{ 
                stream: Some(stream),
                pool: Arc::clone(self)
            });
        }
        
        
        Err(std::io::Error::new(ErrorKind::WouldBlock, "max connection limits hit"))
    }


    pub async fn is_pool_exhausted(self: &Arc<Self>) -> bool { 
        let inner = self.inner.lock().await;
        inner.idle.is_empty() && inner.active > self.max_conns
    }
}


impl PooledConn { 
    pub fn stream(&mut self) -> &mut TcpStream { 
        self.stream.as_mut().unwrap()
    }
}

impl Drop for PooledConn {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() { 
            let pool = self.pool.clone();
            tokio::spawn(async move { 
                println!("dropping");
                let mut inner = pool.inner.lock().await;
                if inner.active > 0 { inner.active -= 1; }
                inner.idle.push(stream);
                println!("after drop connections stacks: {}", inner.idle.len());
                drop(inner);
            });
        }
    }
}

