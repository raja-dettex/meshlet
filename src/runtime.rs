use tokio::runtime::Runtime;

pub fn init() -> Runtime {
    tokio::runtime::Builder
        ::new_current_thread().worker_threads(2)
        .enable_all().build().expect("failed to build meshlet runtime")
}