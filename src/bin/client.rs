use std::{net::SocketAddr, sync::Arc};

use quinn::{ClientConfig, Endpoint};
use rustls::pki_types::{pem::PemObject, CertificateDer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert = CertificateDer::from_pem_file("cert.pem")?;
    let mut certs = rustls::RootCertStore::empty();
    certs.add(cert)?;

    let client_config = ClientConfig::with_root_certificates(Arc::new(certs))?;

    let endpoint = {
        let bind_addr: SocketAddr = "[::]:0".parse()?;
        let mut endpoint = Endpoint::client(bind_addr)?;
        endpoint.set_default_client_config(client_config);
        endpoint
    };

    let server_addr: SocketAddr = "[::1]:1234".parse()?;

    let new_conn = endpoint.connect(server_addr, "hello.world.example")?.await?;
    let (mut w, mut r) = new_conn.open_bi().await?;
    tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        let _ = tokio::io::copy(&mut r, &mut stdout).await;
    });
    let mut stdin = tokio::io::stdin();
    tokio::io::copy(&mut stdin, &mut w).await?;
    w.finish()?;
    new_conn.close(0u32.into(), b"done");
    endpoint.wait_idle().await;
    Ok(())
}