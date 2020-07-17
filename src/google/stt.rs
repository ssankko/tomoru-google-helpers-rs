crate::service!("speech", "https://www.googleapis.com/auth/cloud-platform");
pub use super::generated::google::cloud::speech::v1::RecognitionConfig;
use super::generated::google::cloud::speech::v1::*;

fn default_config() -> RecognitionConfig {
    RecognitionConfig {
        // encoding: Linear16
        encoding: 1,
        // FIXME pass format
        sample_rate_hertz: 8000,
        audio_channel_count: 1,
        enable_separate_recognition_per_channel: false,
        language_code: "ru".to_string(),
        // return at most one hyphothesis at the end of recognition
        max_alternatives: 0,
        profanity_filter: false,
        // no contexts for now
        speech_contexts: vec![],
        enable_word_time_offsets: false,
        enable_automatic_punctuation: false,
        diarization_config: None,
        metadata: None,
        model: Default::default(),
        use_enhanced: true,
    }
}

pub async fn recognize(uri: String, config: Option<RecognitionConfig>) -> Option<String> {
    let stt = SERVICE.get().unwrap();
    let config = config.unwrap_or_else(default_config);
    // --------------------------------
    // construct request
    // --------------------------------
    let request = RecognizeRequest {
        config: Some(config),
        audio: Some(RecognitionAudio {
            audio_source: Some(recognition_audio::AudioSource::Uri(uri)),
        }),
    };

    // --------------------------------
    // retrieve token and construct channel
    // --------------------------------
    let channel = stt.channel.clone();
    let token = stt.auth.token(SCOPES).await.unwrap();
    let bearer_token = format!("Bearer {}", token.as_str());
    let token = MetadataValue::from_str(&bearer_token).unwrap();

    let mut service =
        speech_client::SpeechClient::with_interceptor(channel, move |mut req: Request<()>| {
            let token = token.clone();
            req.metadata_mut().insert("authorization", token);
            Ok(req)
        });

    // --------------------------------
    // send request
    // --------------------------------
    let response = service
        .recognize(Request::new(request))
        .await
        .unwrap()
        .into_inner();

    // --------------------------------
    // take required result
    // --------------------------------
    response
        .results
        .get(0)
        .and_then(|x| x.alternatives.get(0))
        .map(|x| x.transcript.clone())
}
