use std::{fs::File, io::Write};

use rcgen::CertifiedKey;

fn main() -> std::io::Result<()> {
    let subject_alt_names = vec!["hello.world.example".to_string()];
    let CertifiedKey { cert, key_pair } =
        rcgen::generate_simple_self_signed(subject_alt_names).unwrap();

    let mut file = File::create("cert.pem")?;
    file.write_all(cert.pem().as_bytes())?;
    file.flush()?;

    let mut file = File::create("key.pem")?;
    file.write_all(key_pair.serialize_pem().as_bytes())?;
    file.flush()?;
    Ok(())
}
