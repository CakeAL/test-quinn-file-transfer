use std::{io, net::SocketAddr, time::Duration};

use tokio::{net::UdpSocket, sync::watch, time};

pub async fn send_udp_packet(from: SocketAddr, to: SocketAddr) -> io::Result<()> {
    let (tx, mut rx) = watch::channel(false);
    let socket = UdpSocket::bind(from).await?;
    let mut buf = [0u8; 1024];

    println!("send on: {:?}", socket.local_addr());
    loop {
        tokio::select! {
                _ = time::sleep(Duration::from_secs(1)) => {
                     socket.send_to(b"hello", to).await.unwrap();
                }
                _ = rx.changed() => {
                    break;
                }
                n = socket.recv(&mut buf) => {
                    let n = n?;
                    let s = String::from_utf8_lossy(&buf[..n]);
                    if s == "hello" {
                        socket.send_to(b"ok", to).await?;
                    } else if s == "ok" {
                        let _ = tx.send(true);
                        break;
                    }
                }
            }
    }
    Ok(())
}
