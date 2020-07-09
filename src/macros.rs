#[macro_export]
macro_rules! service {
    ($domain_name: literal, $($scope: literal),+) => {
        use crate::{auth, Service};
        use once_cell::sync::OnceCell;
        use tonic::{
            metadata::MetadataValue,
            transport::{Channel, ClientTlsConfig},
            Request,
        };

        const DEFAULT_HOST: &str = concat!("https://", $domain_name, ".googleapis.com");
        const SCOPES: &[&str] = &[$($scope),+];

        static SERVICE: OnceCell<Service> = OnceCell::new();

        pub(super) async fn initialize<'a>(
            tls_config: ClientTlsConfig,
            key: &str,
        ) {
            let inner = Service {
                auth: auth(key, SCOPES).await,
                channel: Channel::from_shared(DEFAULT_HOST)
                    .unwrap()
                    .tls_config(tls_config)
                    .connect()
                    .await
                    .unwrap(),
            };
            if let Err(_) = SERVICE.set(inner) {
                panic!(concat!("Already initialized ", $domain_name, " service"));
            }
        }
    };
}
