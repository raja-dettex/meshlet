
use crate::config::Config;

pub mod runtime;
pub mod listener;
pub mod config;
pub mod conn;


fn main() -> anyhow::Result<()>{
    let rt = runtime::init();
    let config = Config::from_env()?;
    rt.block_on(async { 
        let _ = listener::run(config).await;
    });
    anyhow::Ok(())
}
