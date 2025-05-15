use actix_web::HttpRequest;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Keys {
    keys: Vec<Key>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Key {
    n: String,
    e: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // ID token equal to client ID
    aud: String,
    // expiry time
    exp: usize,
    // accounts.google.com or https://accounts.google.com
    iss: String,
    // userid
    pub sub: String,
}

pub async fn verify_login(req: HttpRequest, client_id: String) -> Result<Claims, String> {
    let keys = get_keys().await;
    let id_token = req.headers().get("Id-Token").unwrap().to_str().unwrap();

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&client_id]);

    for key in keys {
        match decode::<Claims>(
            id_token,
            &DecodingKey::from_rsa_components(&key.n, &key.e).unwrap(),
            &validation,
        ) {
            Ok(token) => {
                if token.claims.aud != client_id {
                    return Err("Invalid client_id".to_string());
                }

                let valid_iss = vec![
                    "accounts.google.com".to_string(),
                    "https://accounts.google.com".to_string(),
                ];

                if !valid_iss.contains(&token.claims.iss) {
                    return Err("Invalid iss".to_string());
                }

                return Ok(token.claims);
            }
            Err(err) => {
                println!("decoding failed: {err:?}");
            }
        }
    }

    Err("No working key found".to_string())
}

pub async fn get_keys() -> Vec<Key> {
    reqwest::get("https://www.googleapis.com/oauth2/v3/certs")
        .await
        .unwrap()
        .json::<Keys>()
        .await
        .unwrap()
        .keys
}
