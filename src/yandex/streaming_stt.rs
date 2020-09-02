use crate::yandex::generated::yandex::cloud::ai::stt::v2;

use super::Service;
use once_cell::sync::OnceCell;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tonic::{
    metadata::MetadataValue,
    transport::{Channel, ClientTlsConfig},
    Request,
};
pub use v2::{RecognitionConfig, RecognitionSpec, StreamingRecognitionResponse};

const DEFAULT_HOST: &str = concat!("https://", "stt.api.cloud", ".yandex.net");

static SERVICE: OnceCell<Service> = OnceCell::new();

pub(crate) async fn initialize<'a>(tls_config: ClientTlsConfig, folder_id: &'static str) {
    let inner = Service {
        channel: Channel::from_shared(DEFAULT_HOST)
            .unwrap()
            .tls_config(tls_config)
            .unwrap()
            .connect()
            .await
            .unwrap(),
        folder_id,
    };
    if SERVICE.set(inner).is_err() {
        panic!("Already initialized stt.api.cloud service");
    }
}

fn default_config(folder_id: String) -> v2::RecognitionConfig {
    v2::RecognitionConfig {
        specification: Some(v2::RecognitionSpec {
            audio_encoding: v2::recognition_spec::AudioEncoding::Linear16Pcm as i32,
            sample_rate_hertz: 8000,
            language_code: "ru-RU".to_owned(),
            profanity_filter: false,
            model: "general:rc".to_owned(),
            partial_results: true,
            single_utterance: false,
            audio_channel_count: 1,
            raw_results: true,
        }),
        folder_id,
    }
}

pub async fn streaming_recognize(
    config: Option<v2::RecognitionConfig>,
) -> (
    UnboundedSender<Vec<u8>>,
    UnboundedReceiver<StreamingRecognitionResponse>,
) {
    let stt = SERVICE.get().unwrap();
    let config = config.unwrap_or_else(|| default_config(stt.folder_id.to_string()));

    // --------------------------------
    // retrieve token and construct channel
    // --------------------------------
    let channel = stt.channel.clone();
    let token = super::auth::get_auth_token().await;
    let bearer_token = format!("Bearer {}", token.as_str());
    let token = MetadataValue::from_str(&bearer_token).unwrap();

    let mut service = v2::stt_service_client::SttServiceClient::with_interceptor(
        channel,
        move |mut req: Request<()>| {
            let token = token.clone();
            req.metadata_mut().insert("authorization", token);
            Ok(req)
        },
    );

    let (audio_sender, mut audio_receiver) = tokio::sync::mpsc::unbounded_channel();

    let stream = async_stream::stream! {
        // config first
        yield v2::StreamingRecognitionRequest {
            streaming_request: Some(v2::streaming_recognition_request::StreamingRequest::Config(
                config,
            )),
        };;

        while let Some(audio) = audio_receiver.recv().await {
            yield v2::StreamingRecognitionRequest {
                streaming_request: Some(
                    v2::streaming_recognition_request::StreamingRequest::AudioContent(audio),
                ),
            };
        }
    };

    let (result_sender, result_receiver) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        let messages = service.streaming_recognize(stream).await.unwrap();
        let mut inner = messages.into_inner();
        while let Some(message) = inner.message().await.unwrap() {
            result_sender.send(message).unwrap();
        }
    });

    (audio_sender, result_receiver)
}
