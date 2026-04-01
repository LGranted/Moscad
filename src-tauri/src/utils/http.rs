use hyper::{Client, Request, Body};
use std::path::PathBuf;
use tokio::net::UnixStream;
use tower::Service;
use std::task::{Context, Poll};
use std::pin::Pin;
use futures::future::BoxFuture;

#[derive(Clone)]
pub struct UnixConnector {
    pub path: PathBuf,
}

impl Service<hyper::Uri> for UnixConnector {
    type Response = UnixStream;
    type Error = std::io::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _dst: hyper::Uri) -> Self::Future {
        let path = self.path.clone();
        Box::pin(async move { UnixStream::connect(path).await })
    }
}

pub fn get_client() -> Client<UnixConnector, Body> {
    let connector = UnixConnector {
        path: PathBuf::from("/data/data/com.lbjlaq.antigravity_tools/files/utls.sock"),
    };
    Client::builder().build(connector)
}

// Заглушка для совместимости со старым кодом
pub fn get_long_standard_client() -> Client<UnixConnector, Body> {
    get_client()
}
