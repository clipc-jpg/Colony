


use reqwest::{Client, Certificate, Identity};


fn test_mtls() -> Result<(), Box<dyn std::error::Error>> {
    // Load client certificate and private key (in PKCS#12 or PEM format)
    let cert = std::fs::read("client-cert.pem")?;
    let key = std::fs::read("client-key.pem")?;
    let identity = Identity::from_pem(&[cert, key].concat())?;

    // Load the CA certificate to trust the server
    let ca_cert = std::fs::read("ca-cert.pem")?;
    let ca_cert = Certificate::from_pem(&ca_cert)?;

    // Build the client
    let client = Client::builder()
    .identity(identity) // client cert + key
    .add_root_certificate(ca_cert) // trust for server cert
    .use_rustls_tls()
    .build()?;

    // Make a request to your Actix server
    let res = client
    .get("https://localhost:8443/")
    .send()
    .await?;

    println!("Status: {}", res.status());
    let body = res.text().await?;
    println!("Body: {}", body);

    Ok(())
}










