#[macro_export]
macro_rules! service {
    ($domain_name: literal, $($scope: literal),+) => {
        use crate::google::{auth, Service};
        use once_cell::sync::OnceCell;
        use tonic::{
            metadata::MetadataValue,
            transport::{Channel, ClientTlsConfig},
            Request,
        };

        const DEFAULT_HOST: &str = concat!("https://", $domain_name, ".googleapis.com");
        const SCOPES: &[&str] = &[$($scope),+];

        static SERVICE: OnceCell<Service> = OnceCell::new();

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
            let inner = Service { channel, auth };
            if SERVICE.set(inner).is_err() {
                panic!(concat!("Already initialized ", $domain_name, " service"));
            }
        }
    };
}
