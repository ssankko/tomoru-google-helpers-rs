#[cfg(feature = "_rpc")]
use tonic::transport::{Channel, ClientTlsConfig};

mod auth;
mod generated;
#[cfg(feature = "yandex-streaming-stt")]
pub mod streaming_stt;
#[cfg(feature = "yandex-stt")]
pub mod stt;

#[cfg(feature = "_rpc")]
pub struct RpcBuilder {
    tls_config: ClientTlsConfig,
    folder_id: String,
}

macro_rules! initialize_fn {
    ($name: ident, $fun_name: ident) => {
        pub async fn $fun_name(self) -> RpcBuilder {
            $name::initialize(self.tls_config.clone(), self.folder_id.clone()).await;
            self
        }
    };
}

#[cfg(feature = "_rpc")]
impl RpcBuilder {
    pub async fn new(key: &[u8], folder_id: String) -> RpcBuilder {
        let mut tls_config = tokio_rustls::rustls::ClientConfig::new();
        tls_config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        tls_config.set_protocols(&["h2".into()]);
        let tls_config = ClientTlsConfig::new().rustls_client_config(tls_config);

        auth::initialize_auth(key).await;

        RpcBuilder {
            tls_config,
            folder_id,
        }
    }

    #[cfg(feature = "yandex-streaming-stt")]
    initialize_fn!(streaming_stt, initialize_streaming_stt);
    // #[cfg(feature = "google-tts")]
    // initialize_fn!(tts, initialize_tts);
    // #[cfg(feature = "google-tasks")]
    // initialize_fn!(tasks, initialize_tasks);
    // #[cfg(feature = "google-logging")]
    // initialize_fn!(logging, initialize_logging);
}

#[cfg(feature = "_rpc")]
struct Service {
    channel: Channel,
    folder_id: String,
}
