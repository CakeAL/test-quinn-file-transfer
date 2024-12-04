use std::{net::SocketAddr, sync::Arc};

use stun::{agent::TransactionId, client::ClientBuilder, message::{Getter, Message, BINDING_REQUEST}, xoraddr::XorMappedAddress};
use tokio::net::UdpSocket;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stun_server: SocketAddr = "[2600:1f16:8c5:101:80b:b58b:828:8df4]:3478".parse()?;
    let conn = UdpSocket::bind("[::]:0").await?;
    conn.connect(stun_server).await?;
    let mut client = ClientBuilder::new().with_conn(Arc::new(conn)).build()?;
    
    let mut msg = Message::new();
    msg.build(&[Box::<TransactionId>::default(), Box::new(BINDING_REQUEST)])?;
    
    let (handler_tx, mut handler_rx) = tokio::sync::mpsc::unbounded_channel();
    client.send(&msg, Some(Arc::new(handler_tx))).await?;

    if let Some(event) = handler_rx.recv().await {
        let msg = event.event_body?;
        let mut xor_addr = XorMappedAddress::default();
        xor_addr.get_from(&msg)?;
        println!("Got response: {xor_addr}");
    }
    client.close().await?;
    Ok(())
}