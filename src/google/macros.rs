#[macro_export]
macro_rules! rpc_service {
    ($domain_name: literal, $($scope: literal),+) => {
        use crate::google::{auth};
        use once_cell::sync::OnceCell;
        use tonic::{
            metadata::MetadataValue,
            transport::{Channel, ClientTlsConfig},
            Request,
        };
        use yup_oauth2::authenticator::DefaultAuthenticator;

        const DEFAULT_HOST: &str = concat!("https://", $domain_name, ".googleapis.com");
        const SCOPES: &[&str] = &[$($scope),+];

        struct RpcService {
            channel: Channel,
            auth: DefaultAuthenticator,
        }

        static SERVICE: OnceCell<RpcService> = OnceCell::new();

        pub(crate) async fn initialize<'a>(
            tls_config: ClientTlsConfig,
            key: &str,
        ) {
            let channel = Channel::from_shared(DEFAULT_HOST)
                .unwrap()
                .tls_config(tls_config)
                .unwrap()
                .connect()
                .await
                .unwrap();
            let auth = auth(key, SCOPES).await;
            let inner = RpcService { channel, auth };
            if SERVICE.set(inner).is_err() {
                panic!(concat!("Already initialized ", $domain_name, " service"));
            }
        }
    };
}

#[macro_export]
macro_rules! rest_service {
    ($domain_name: literal, $($scope: literal),+) => {
        use crate::google::{auth};
        use once_cell::sync::OnceCell;
        use reqwest::Client;
        use yup_oauth2::authenticator::DefaultAuthenticator;

        const SCOPES: &[&str] = &[$($scope),+];

        struct RestService {
            client: Client,
            auth: DefaultAuthenticator,
        }

        static SERVICE: OnceCell<RestService> = OnceCell::new();

        pub(crate) async fn initialize<'a>(
            key: &str,
        ) {
            let client = Client::builder().timeout(std::time::Duration::from_secs(60)).build().unwrap();
            let auth = auth(key, SCOPES).await;
            let inner = RestService { client, auth };
            if SERVICE.set(inner).is_err() {
                panic!(concat!("Already initialized ", $domain_name, " service"));
            }
        }
    };
}
