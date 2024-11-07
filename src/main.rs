use anyhow::Result;
use tracing::{error, info, info_span, Instrument};
mod http;
mod utils;
mod webtransport;
use http::HttpServer;
use webtransport::WebTransportServer;
use wtransport::Identity;

#[tokio::main]
async fn main() -> Result<()> {
    utils::init_logging();

    let identity = Identity::self_signed(["localhost", "127.0.0.1", "::1"]).unwrap();
    let cert_digest = identity.certificate_chain().as_slice()[0].hash();

    let webtransport_server = WebTransportServer::new(identity)?;
    let http_server = HttpServer::new(&cert_digest, webtransport_server.local_port()).await?;

    info!(
        "Open the browser and go to: http://127.0.0.1:{}",
        http_server.local_port()
    );

    tokio::select! {
        result = http_server.serve() => {
            error!("HTTP server: {:?}", result);
        }
        result = webtransport_server.serve() => {
            error!("WebTransport server: {:?}", result);
        }
    }

    Ok(())
}
