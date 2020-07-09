crate::service!(
    "texttospeech",
    "https://www.googleapis.com/auth/cloud-platform"
);

use crate::generated::google::cloud::texttospeech::v1::*;

pub async fn synthesize(phrase: String) -> Result<Vec<u8>, tonic::Status> {
    let tts = SERVICE.get().unwrap();

    // --------------------------------
    // construct request
    // --------------------------------
    println!("[GRPC SYNTHESIZE] trying to synth phrase: {}", phrase);
    let request = SynthesizeSpeechRequest {
        audio_config: Some(AudioConfig {
            audio_encoding: 1,
            speaking_rate: 1.2,
            pitch: 1.0,
            volume_gain_db: 0.0,
            sample_rate_hertz: 8000,
            effects_profile_id: vec![],
        }),
        input: Some(SynthesisInput {
            input_source: Some(synthesis_input::InputSource::Ssml(format!(
                "<speak>{}</speak>",
                phrase
            ))),
        }),
        voice: Some(VoiceSelectionParams {
            language_code: "ru".to_string(),
            name: "ru-RU-Wavenet-C".to_string(),
            // Unspecified
            ssml_gender: 2,
        }),
    };

    // --------------------------------
    // retrieve token and construct channel
    // --------------------------------
    let channel = tts.channel.clone();
    let token = tts.auth.token(SCOPES).await.unwrap();
    let bearer_token = format!("Bearer {}", token.as_str());
    let token = MetadataValue::from_str(&bearer_token).unwrap();

    let mut service = text_to_speech_client::TextToSpeechClient::with_interceptor(
        channel,
        move |mut req: Request<()>| {
            let token = token.clone();
            req.metadata_mut().insert("authorization", token);
            Ok(req)
        },
    );

    // --------------------------------
    // send request
    // --------------------------------
    let response = service.synthesize_speech(request).await;

    // --------------------------------
    // take required result
    // --------------------------------
    response.map(|x| x.into_inner().audio_content)
}
