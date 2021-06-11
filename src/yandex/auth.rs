use jsonwebtoken::{encode, EncodingKey, Header};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize)]
struct Claims<'t> {
    iss: &'t str,
    aud: &'t str,
    iat: u64,
    exp: u64,
}

static YANDEX_KEY: OnceCell<EncodingKey> = OnceCell::new();

pub(super) async fn initialize_auth(key: &[u8]) {
    YANDEX_KEY
        .set(EncodingKey::from_rsa_pem(key).unwrap())
        .unwrap();
    get_auth_token().await;
}

#[derive(Serialize)]
struct TokenRequestPayload {
    jwt: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenRequestResult {
    iam_token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

async fn get_iam_token() -> TokenRequestResult {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let hour_later = now + 3600;

    let mut h = Header::new(jsonwebtoken::Algorithm::PS256);
    h.kid = Some("aje04ppj0e85d7njj0sf".to_owned());

    let claims = Claims {
        iss: "ajede2r7i8dtgcgehtdl",
        aud: "https://iam.api.cloud.yandex.net/iam/v1/tokens",
        iat: now,
        exp: hour_later,
    };
    let token = encode(&h, &claims, &YANDEX_KEY.get().unwrap()).unwrap();

    let result = reqwest::Client::new()
        .post(
            "https://iam.api.cloud.yandex.net/iam/v1/tokens"
                .parse::<reqwest::Url>()
                .unwrap(),
        )
        .json(&TokenRequestPayload { jwt: token })
        .send()
        .await
        .unwrap();

    result.json::<TokenRequestResult>().await.unwrap()
}

static TOKEN: OnceCell<RwLock<TokenRequestResult>> = OnceCell::new();

pub async fn get_auth_token() -> String {
    if let Some(res) = TOKEN.get() {
        {
            let lock = res.read().await;
            if lock.expires_at - chrono::Utc::now() > chrono::Duration::zero() {
                return lock.iam_token.clone();
            }
        }
        let result = get_iam_token().await;
        let token = result.iam_token.clone();
        *res.write().await = result;
        token
    } else {
        let result = get_iam_token().await;
        let token = result.iam_token.clone();
        let _ = TOKEN.set(RwLock::new(result));
        token
    }
}
