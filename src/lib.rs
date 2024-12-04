use std::{io, net::SocketAddr, time::Duration};

use tokio::{net::UdpSocket, sync::watch, time};

pub async fn send_udp_packet(addr: SocketAddr, mut rx: watch::Receiver<bool>) -> io::Result<()> {
    let socket = UdpSocket::bind("[::]:0").await?;
    println!("send on: {:?}", socket.local_addr());
    tokio::spawn(async move {       
        loop {
            tokio::select! {
                _ = time::sleep(Duration::from_secs(1)) => {
                     socket.send_to(b"hello", addr).await.unwrap();
                }
                _ = rx.changed() => {
                    break;
                }
            }
        }
    });
    Ok(())
}

pub async fn listener(addr: SocketAddr, tx: watch::Sender<bool>) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    let socket = UdpSocket::bind("[::]:0").await?;
    loop {
        let n = socket.recv(&mut buf).await?;
        let s = String::from_utf8_lossy(&buf[..n]);
        if s == "hello" {
            socket.send_to(b"ok", addr).await?;
        } else if s == "ok" {
            let _ = tx.send(true);
            break;
        }
    }
    Ok(())
}