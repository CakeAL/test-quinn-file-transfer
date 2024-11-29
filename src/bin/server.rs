use std::net::SocketAddr;

use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert = CertificateDer::from_pem_file("cert.pem")?;
    let key = PrivateKeyDer::from_pem_file("key.pem")?;
    let cert_chain = vec![cert];

    let server_config = ServerConfig::with_single_cert(cert_chain, key)?;
    let bind_addr: SocketAddr = "[::1]:1234".parse()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    let mut buf = [0u8; 1024];

    while let Some(income_conn) = endpoint.accept().await {
        match income_conn.await {
            Ok(new_conn) => {
                tokio::spawn(async move {
                    match new_conn.accept_bi().await {
                        Ok((mut wstream, mut rstream)) => loop {
                            let _len = rstream.read(&mut buf).await.unwrap();
                            if let Some(_len) = _len {
                                let recv = String::from_utf8_lossy(&buf[.._len]);
                                let recv = format!("Recv: {}", recv);
                                eprintln!("{}", recv);
                                wstream.write_all(recv.as_bytes()).await.unwrap();
                            } else {
                                break;
                            }
                        },
                        Err(e) => {
                            eprintln!("{e:?}");
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("{e:?}");
            }
        }
    }
    endpoint.wait_idle().await;
    eprintln!("END");
    Ok(())
}
