use std::path::PathBuf;
use tokio::net::UnixStream;
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
        path: PathBuf::from("/data/data/com.lbjlaq.antigravity/files/utls.sock"),
    };
    Client::builder(TokioExecutor::new()).build(connector)
}

pub fn get_long_standard_client() -> Client<UnixConnector, Full<Bytes>> {
    get_client()
}

// ── Stealth модуль для proxy_android_stub.rs ─────────────────────────────────
// proxy_android_stub использует hyper014, поэтому здесь отдельный модуль

pub mod stealth {
    use std::path::PathBuf;
    use tokio::net::UnixStream;
    use hyper014::client::connect::{Connected, Connection};
    use hyper014::Uri;
    use std::future::Future;
    use std::io;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    pub type StealthClient = hyper014::Client<UnixConnector, hyper014::Body>;

    #[derive(Clone)]
    pub struct UnixConnector {
        path: PathBuf,
    }

    impl tower::Service<Uri> for UnixConnector {
        type Response = UnixStream;
        type Error = io::Error;
        type Future = Pin<Box<dyn Future<Output = io::Result<UnixStream>> + Send + 'static>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _: Uri) -> Self::Future {
            let path = self.path.clone();
            Box::pin(async move { UnixStream::connect(path).await })
        }
    }

    impl Connection for UnixStream {
        fn connected(&self) -> Connected { Connected::new() }
    }

    pub fn get_stealth_client() -> anyhow::Result<StealthClient> {
        get_stealth_client_for(None)
    }

    pub fn get_stealth_client_for(_account_seed: Option<&str>) -> anyhow::Result<StealthClient> {
        let connector = UnixConnector {
            path: PathBuf::from("/data/data/com.lbjlaq.antigravity/files/utls.sock"),
        };
        let client = hyper014::Client::builder()
            .http1_only(true)
            .pool_max_idle_per_host(8)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build::<_, hyper014::Body>(connector);
        Ok(client)
    }
}
