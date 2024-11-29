use std::{io, net::SocketAddr, sync::Arc};

use quinn::{crypto::rustls::QuicClientConfig, Endpoint};
use rustls::client::danger::{ServerCertVerified, ServerCertVerifier};

#[derive(Debug)]
struct TrustOnFirstUseVerifier(Arc<rustls::crypto::CryptoProvider>);

impl TrustOnFirstUseVerifier {
    fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(rustls::crypto::ring::default_provider())))
    }
}

impl ServerCertVerifier for TrustOnFirstUseVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        println!("Certificate: {:?}", end_entity);
        println!("Do you trust this certificate? (y/n)");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().to_lowercase() != "y" {
            println!("Certificate not trusted. Exiting.");
            return Err(quinn::rustls::Error::General(
                "Certificate not trusted".into(),
            ));
        }
        println!("Certificate trusted");
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let cert = CertificateDer::from_pem_file("cert.pem")?;
    // let mut certs = rustls::RootCertStore::empty();
    // certs.add(cert)?;
    // let client_config = ClientConfig::with_root_certificates(Arc::new(certs))?;
    // let mut certs = rustls::RootCertStore::empty();
    // for cert in rustls_native_certs::load_native_certs().certs {
    //     certs.add(cert)?;
    // }
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    let client_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(TrustOnFirstUseVerifier::new())
        .with_no_client_auth();

    let client_config =
        quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_config)?));

    let endpoint = {
        let bind_addr: SocketAddr = "[::]:0".parse()?;
        let mut endpoint = Endpoint::client(bind_addr)?;
        endpoint.set_default_client_config(client_config);
        endpoint
    };

    let server_addr: SocketAddr = "[::1]:1234".parse()?;

    let new_conn = endpoint
        .connect(server_addr, "hello.world.example")?
        .await?;
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
