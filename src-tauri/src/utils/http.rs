use std::path::PathBuf;
use tokio::net::UnixStream;
use hyper::Request;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::Connect;
use hyper_util::rt::TokioExecutor;
use http_body_util::Full;
use bytes::Bytes;
use futures::future::BoxFuture;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct UnixConnector {
    pub path: PathBuf,
}

impl hyper::service::Service<hyper::Uri> for UnixConnector {
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

impl Connect for UnixConnector {
    fn connect(&self, _fut: hyper_util::client::legacy::connect::Connected, _dst: hyper::Uri) -> BoxFuture<'static, Result<(Self::Response, hyper_util::client::legacy::connect::Connected), Self::Error>> {
        let path = self.path.clone();
        Box::pin(async move {
            let stream = UnixStream::connect(path).await?;
            Ok((stream, hyper_util::client::legacy::connect::Connected::new()))
        })
    }
}

pub fn get_client() -> Client<UnixConnector, Full<Bytes>> {
    let connector = UnixConnector {
        path: PathBuf::from("/data/data/com.lbjlaq.antigravity_tools/files/utls.sock"),
    };
    Client::builder(TokioExecutor::new()).build(connector)
}

pub fn get_long_standard_client() -> Client<UnixConnector, Full<Bytes>> {
    get_client()
}
