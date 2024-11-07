mod http_data;

use super::*;
use anyhow::Context;
use axum::http::header::CONTENT_TYPE;
use axum::response::Html;
use axum::routing::get;
use axum::serve;
use axum::serve::Serve;
use axum::Router;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use wtransport::tls::{Sha256Digest, Sha256DigestFmt};

pub struct HttpServer {
    serve: Serve<Router, Router>,
    local_port: u16,
}

impl HttpServer {
    const PORT: u16 = 8080;

    pub async fn new(cert_digest: &Sha256Digest, webtransport_port: u16) -> Result<Self> {
        let router = Self::build_router(cert_digest, webtransport_port);

        let listener = TcpListener::bind(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), Self::PORT))
            .await
            .context("Cannot bind TCP listener for HTTP server")?;

        let local_port = listener
            .local_addr()
            .context("Cannot get local port")?
            .port();

        Ok(HttpServer {
            serve: serve(listener, router),
            local_port,
        })
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    pub async fn serve(self) -> Result<()> {
        info!("Server running on port {}", self.local_port());

        self.serve.await.context("HTTP server error")?;

        Ok(())
    }

    fn build_router(cert_digest: &Sha256Digest, webtransport_port: u16) -> Router {
        let cert_digest = cert_digest.fmt(Sha256DigestFmt::BytesArray);

        let root = move || async move {
            Html(
                http_data::INDEX_DATA
                    .replace("${WEBTRANSPORT_PORT}", &webtransport_port.to_string()),
            )
        };

        let style = move || async move { ([(CONTENT_TYPE, "text/css")], http_data::STYLE_DATA) };

        let client = move || async move {
            (
                [(CONTENT_TYPE, "application/javascript")],
                http_data::CLIENT_DATA.replace("${CERT_DIGEST}", &cert_digest),
            )
        };

        Router::new()
            .route("/", get(root))
            .route("/style.css", get(style))
            .route("/client.js", get(client))
    }
}
