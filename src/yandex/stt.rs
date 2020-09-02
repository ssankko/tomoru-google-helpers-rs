use once_cell::sync::OnceCell;
use reqwest::Client;

lazy_static::lazy_static! {
    static ref YANDEX_SHORT_STT_URL: reqwest::Url = reqwest::Url::parse("https://stt.api.cloud.yandex.net/speech/v1/stt:recognize?topic=general:rc&format=lpcm&sampleRateHertz=8000").unwrap();
}
static CLIENT: OnceCell<Client> = OnceCell::new();

pub async fn recognize(audio: Vec<u8>) -> Option<String> {
    let client = CLIENT.get_or_init(Client::new);

    let result = client
        .post(YANDEX_SHORT_STT_URL.clone())
        .bearer_auth(super::auth::get_auth_token().await)
        .body(audio)
        .send()
        .await
        .ok()?;

    let recognized = result
        .json::<serde_json::Value>()
        .await
        .ok()?
        .get("result")?
        .as_str()?
        .to_owned();

    Some(recognized)
}
