use actix_web::{
    body::BoxBody, get, http::header::ContentType, middleware::Logger, post, web, App, HttpRequest,
    HttpResponse, HttpServer, Responder,
};
use env_logger::Env;
use firestore::FirestoreDb;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};


const RANKINGS_COLLECTION: &'static str = "rankings";
const ENDRESULT_COLLECTION: &'static str = "endresult";
const USER_COLLECTION: &'static str = "user";
const ENDRESULT_ID: &'static str = "endresult_id";

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Ranking {
    name: String,
    countries: Vec<String>,
}

impl Responder for Ranking {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[post("/ranking")]
async fn post_ranking(ranking: web::Json<Ranking>, data: web::Data<AppState>) -> impl Responder {
    let r = ranking.0;

    data.db
        .fluent()
        .update()
        .in_col(RANKINGS_COLLECTION)
        .document_id(&r.name)
        .object(&r)
        .execute::<Ranking>()
        .await
        .unwrap();

    HttpResponse::Ok().finish()
}

#[get("/ranking")]
async fn get_ranking(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let id_token = req.headers().get("Id-Token").unwrap().to_str().unwrap();
    let key = data.gso_keys.first().unwrap();
    let token = decode::<Claims>(
        &id_token,
        &DecodingKey::from_rsa_components(&key.n, &key.e).unwrap(),
        &Validation::new(Algorithm::RS256),
    )
    .unwrap();

    todo!("Somehow look up by id here...");

    /*
    let ranking = data
        .db
        .fluent()
        .select()
        .by_id_in(RANKINGS_COLLECTION)
        .obj::<Ranking>()
        .one(name)
        .await
        .unwrap()
        .expect("ranking not found");

    let body = serde_json::to_string(&ranking).unwrap();
    HttpResponse::Ok().body(body)
    */
    HttpResponse::Ok()
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct User {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String,
    iss: String,
    exp: usize,
    sub: String,
}

#[post("/user")]
async fn post_user(
    user: web::Json<User>,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let id_token = req.headers().get("Id-Token").unwrap().to_str().unwrap();

    let key = data.gso_keys.first().unwrap();
    let token = decode::<Claims>(
        &id_token,
        &DecodingKey::from_rsa_components(&key.n, &key.e).unwrap(),
        &Validation::new(Algorithm::RS256),
    )
    .unwrap();

    data.db
        .fluent()
        .update()
        .in_col(USER_COLLECTION)
        .document_id(&token.claims.sub)
        .object(&user.0)
        .execute::<User>()
        .await
        .unwrap();

    HttpResponse::Ok()
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct EndResult {
    countries: Vec<String>,
}

impl Responder for EndResult {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[get("/result")]
async fn result(data: web::Data<AppState>) -> impl Responder {
    let result = data
        .db
        .fluent()
        .select()
        .by_id_in(ENDRESULT_COLLECTION)
        .obj::<EndResult>()
        .one(ENDRESULT_ID.to_string())
        .await
        .unwrap()
        .expect("ranking not found");

    let body = serde_json::to_string(&result).unwrap();
    HttpResponse::Ok().body(body)
}

#[derive(Clone)]
struct AppState {
    db: FirestoreDb,
    gso_keys: Vec<Key>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let port = match std::env::var("PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(_) => 8080,
    };

    let appstate = {
        let db = FirestoreDb::new("esc-api-384517").await.unwrap();

        let gso_keys = reqwest::get("https://www.googleapis.com/oauth2/v3/certs")
            .await.unwrap().json::<Keys>().await.unwrap().keys;


        AppState { db: db, gso_keys: gso_keys }
    };

    println!("Starting esc-api on port {}...", port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(appstate.clone()))
            .service(post_user)
            .service(post_ranking)
            .service(get_ranking)
            .service(result)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Keys {
    keys: Vec<Key>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Key {
    n: String,
    e: String,
}
