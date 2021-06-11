mod generated;
mod macros;
#[cfg(feature = "google-stt")]
pub mod stt;

#[cfg(feature = "google-tasks")]
pub mod tasks;

#[cfg(feature = "google-tts")]
pub mod tts;

#[cfg(feature = "google-logging")]
pub mod logging;

#[cfg(feature = "google-spreadsheets")]
pub mod spreadsheets;

use tonic::transport::ClientTlsConfig;
use yup_oauth2::{authenticator::DefaultAuthenticator, ServiceAccountAuthenticator};

pub struct RpcBuilder<'a> {
    tls_config: ClientTlsConfig,
    key: &'a str,
}

macro_rules! initialize_fn {
    ($name: ident, $fun_name: ident) => {
        pub async fn $fun_name(self) -> RpcBuilder<'a> {
            $name::initialize(self.tls_config.clone(), self.key).await;
            self
        }
    };
}

impl<'a> RpcBuilder<'a> {
    pub fn new(key: &'a str) -> RpcBuilder {
        let mut tls_config = tokio_rustls::rustls::ClientConfig::new();
        tls_config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        tls_config.set_protocols(&["h2".into()]);
        let tls_config = ClientTlsConfig::new().rustls_client_config(tls_config);

        RpcBuilder { tls_config, key }
    }

    #[cfg(feature = "google-stt")]
    initialize_fn!(stt, initialize_stt);
    #[cfg(feature = "google-tts")]
    initialize_fn!(tts, initialize_tts);
    #[cfg(feature = "google-tasks")]
    initialize_fn!(tasks, initialize_tasks);
    #[cfg(feature = "google-logging")]
    initialize_fn!(logging, initialize_logging);
    #[cfg(feature = "google-spreadsheets")]
    pub async fn initialize_spreadsheets(self) -> RpcBuilder<'a> {
        spreadsheets::initialize(self.key).await;
        self
    }
}

async fn auth(key: &str, scopes: &[&str]) -> DefaultAuthenticator {
    let key = serde_json::from_str(key).unwrap();

    let auth = ServiceAccountAuthenticator::builder(key)
        .build()
        .await
        .unwrap();

    // Беру токен, чтобы прогреть, по возможности.
    // Плюс если появятся какие то ошибки, то они будут видны на старте
    let _ = auth.token(scopes).await.unwrap();
    auth
}
