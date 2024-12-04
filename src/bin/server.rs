use std::net::SocketAddr;

use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use test_quinn_file_transfer::send_udp_packet;
use tokio::{fs::File, io::AsyncReadExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert = CertificateDer::from_pem_file("cert.pem")?;
    let key = PrivateKeyDer::from_pem_file("key.pem")?;
    let cert_chain = vec![cert];
    
    let server_config = ServerConfig::with_single_cert(cert_chain, key)?;

    let mut message_buf = [0u8; 1024];
    let bind_addr: SocketAddr = "[::]:23334".parse()?;

    let server_addr: SocketAddr = "[::1]:23333".parse()?;
    send_udp_packet(bind_addr, server_addr).await?;
    println!("established connection");

    let endpoint = Endpoint::server(server_config, bind_addr)?;
    dbg!(&endpoint.local_addr());
    while let Some(income_conn) = endpoint.accept().await {
        match income_conn.await {
            Ok(new_conn) => {
                tokio::spawn(async move {
                    match new_conn.accept_bi().await {
                        Ok((mut wstream, mut rstream)) => loop {
                            let len = rstream.read(&mut message_buf).await.unwrap();
                            if let Some(len) = len {
                                let recv = String::from_utf8_lossy(&message_buf[..len]);
                                println!("accept message: {:?}", recv);
                                let mut file = File::open("/Users/cakeal/Downloads/（带弹幕）【补档】影视飓风 - 技术进步了，画质怎么变差了？.mp4").await.unwrap();
                                let mut buffer = vec![0u8; 16 * 1024];
                                while let Ok(n) = file.read(&mut buffer).await {
                                    if n == 0 {
                                        break;
                                    }
                                    wstream.write_all(&buffer[..n]).await.unwrap();
                                }
                                wstream.finish().unwrap();
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
