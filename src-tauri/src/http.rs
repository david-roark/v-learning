use std::sync::OnceLock;

use crate::error::{AppError, AppResult};

static TLS_CONFIG: OnceLock<rustls::ClientConfig> = OnceLock::new();

fn shared_tls_config() -> rustls::ClientConfig {
    TLS_CONFIG
        .get_or_init(|| {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let mut cfg = rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
            cfg
        })
        .clone()
}

/// Builds a `reqwest::Client` using `webpki-roots` Mozilla CA certificates.
///
/// This avoids relying on `rustls-platform-verifier`'s Android JNI initialization,
/// which otherwise causes every HTTPS request on Android to fail TLS verification.
pub fn build_client(timeout_secs: u64) -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .tls_backend_preconfigured(shared_tls_config())
        .connect_timeout(std::time::Duration::from_secs(timeout_secs.min(8)))
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| AppError::Internal(format!("Không thể tạo HTTP client: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_client_with_webpki_roots_succeeds_and_fetches_mock_http() {
        let client = build_client(10).expect("build_client");
        let (url, _) =
            crate::http_mock::spawn_one_shot("200 OK", "text/plain", b"ok".to_vec());
        let body = tauri::async_runtime::block_on(async move {
            client.get(&url).send().await?.text().await
        })
        .expect("fetch");
        assert_eq!(body, "ok");
    }
}
