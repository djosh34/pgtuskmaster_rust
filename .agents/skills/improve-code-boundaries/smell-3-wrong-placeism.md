# Smell 3: Wrong Place-ism

This smell is about knowledge living in the wrong module.

The classic bad shape is:

- A talks to B
- B talks back to A
- both pass similar request or state types around
- the top-level runtime becomes the courier for everyone else's internals

This is closely related to bad config boundaries. When validation has not been finished, raw or half-validated config tends to spray through runtime, worker startup, and helper functions, and each layer starts compensating in its own way.


e.g. tls.rs not within api


This file
```rust
pub(crate) fn build_api_server_config_v2(
    transport: &ApiTransportV2,
) -> Result<Arc<rustls::ServerConfig>, TlsConfigError> {
    let ApiTransportV2::Https {
        tls,
        client_ca,
        client_cert_required,
        allowed_client_common_names,
    } = transport
    else {
        return Err(TlsConfigError::Rustls {
            message: "https transport required for rustls server config".to_string(),
        });
    };
    let verifier = build_client_verifier_from_paths(
        client_ca.as_deref(),
        *client_cert_required,
        allowed_client_common_names,
    )?;
    let mut config = build_server_config_from_paths(&tls.cert, &tls.key, verifier)?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(Arc::new(config))
}

```

is not inside api? why not? 
it could be pub(super) and inside only api/
having this shared is an antipattern and smell
