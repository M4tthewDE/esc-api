use std::{collections::HashMap, fs::File, io::Read};

use actix_web::{
    body::BoxBody, get, http::header::ContentType, middleware::Logger, post, web, App, HttpRequest,
    HttpResponse, HttpServer, Responder,
};
use env_logger::Env;
use firestore::FirestoreDb;
use serde::{Deserialize, Serialize};

mod auth;

const RANKINGS_COLLECTION: &str = "rankings";
const ENDRESULT_COLLECTION: &str = "endresult";
const USER_COLLECTION: &str = "user";
const LOCK_COLLECTION: &str = "lock";

const ENDRESULT_ID: &str = "endresult_id";
const LOCK_ID: &str = "lock_id";

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Ranking {
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
async fn post_ranking(
    ranking: web::Json<Ranking>,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let claims = auth::verify_login(req, data.client_id.clone())
        .await
        .unwrap();
    let r = ranking.0;

    data.db
        .fluent()
        .update()
        .in_col(RANKINGS_COLLECTION)
        .document_id(&claims.sub)
        .object(&r)
        .execute::<Ranking>()
        .await
        .unwrap();

    HttpResponse::Ok().finish()
}

#[get("/ranking")]
async fn get_ranking(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let claims = auth::verify_login(req, data.client_id.clone())
        .await
        .unwrap();

    let ranking = match data
        .db
        .fluent()
        .select()
        .by_id_in(RANKINGS_COLLECTION)
        .obj::<Ranking>()
        .one(claims.sub)
        .await
    {
        Ok(ranking) => match ranking {
            Some(r) => r,
            None => get_default_ranking(),
        },
        Err(_) => get_default_ranking(),
    };

    let body = serde_json::to_string(&ranking).unwrap();
    HttpResponse::Ok().body(body)
}

fn get_default_ranking() -> Ranking {
    let mut file = File::open("countries.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let countries = serde_json::from_str::<Vec<String>>(&data).unwrap();
    Ranking { countries }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct User {
    name: String,
}

#[post("/user")]
async fn post_user(
    user: web::Json<User>,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let claims = auth::verify_login(req, data.client_id.clone())
        .await
        .unwrap();

    data.db
        .fluent()
        .update()
        .in_col(USER_COLLECTION)
        .document_id(&claims.sub)
        .object(&user.0)
        .execute::<User>()
        .await
        .unwrap();

    HttpResponse::Ok()
}

#[get("/user")]
async fn get_user(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let claims = auth::verify_login(req, data.client_id.clone())
        .await
        .unwrap();

    return match data
        .db
        .fluent()
        .select()
        .by_id_in(USER_COLLECTION)
        .obj::<User>()
        .one(claims.sub)
        .await
        .unwrap()
    {
        Some(user) => {
            let body = serde_json::to_string(&user).unwrap();
            return HttpResponse::Ok().body(body);
        }
        None => HttpResponse::NotFound().finish(),
    };
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct EndResult {
    done: bool,
    countries: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Score {
    score: usize,
    detailed: HashMap<String, usize>,
}

#[get("/score")]
async fn get_score(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let claims = auth::verify_login(req, data.client_id.clone())
        .await
        .unwrap();

    let end_result = data
        .db
        .fluent()
        .select()
        .by_id_in(ENDRESULT_COLLECTION)
        .obj::<EndResult>()
        .one(ENDRESULT_ID.to_string())
        .await
        .unwrap()
        .expect("ranking not found");

    if !end_result.done {
        return HttpResponse::NotFound().finish();
    }

    let user_ranking = match data
        .db
        .fluent()
        .select()
        .by_id_in(RANKINGS_COLLECTION)
        .obj::<Ranking>()
        .one(claims.sub)
        .await
    {
        Ok(ranking) => match ranking {
            Some(r) => r,
            None => get_default_ranking(),
        },
        Err(_) => get_default_ranking(),
    };

    let mut overall_score = 0;
    let mut detailed_score: HashMap<String, usize> = HashMap::new();
    for (index, country) in user_ranking.countries.iter().enumerate() {
        let end_result_index = end_result
            .countries
            .iter()
            .position(|c| c == country)
            .unwrap();

        let diff = match index <= end_result_index {
            true => end_result_index - index,
            false => index - end_result_index,
        };

        let score = 3_usize.checked_sub(diff).or(Some(0)).unwrap();
        detailed_score.insert(country.to_string(), score);
        overall_score += score;
    }

    let score = Score {
        score: overall_score,
        detailed: detailed_score,
    };

    let body = serde_json::to_string(&score).unwrap();
    HttpResponse::Ok().body(body)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Lock {
    lock: bool,
}

#[get("/lock")]
async fn get_lock(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    auth::verify_login(req, data.client_id.clone())
        .await
        .unwrap();

    let lock = data
        .db
        .fluent()
        .select()
        .by_id_in(LOCK_COLLECTION)
        .obj::<Lock>()
        .one(LOCK_ID.to_string())
        .await
        .unwrap()
        .expect("lock not found");

    let body = serde_json::to_string(&lock).unwrap();
    HttpResponse::Ok().body(body)
}

#[post("/lock")]
async fn post_lock(
    lock: web::Json<Lock>,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    auth::verify_login(req, data.client_id.clone())
        .await
        .unwrap();

    data.db
        .fluent()
        .update()
        .in_col(LOCK_COLLECTION)
        .document_id(LOCK_ID.to_string())
        .object(&lock.0)
        .execute::<Lock>()
        .await
        .unwrap();

    let body = serde_json::to_string(&false).unwrap();
    HttpResponse::Ok().body(body)
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[derive(Clone)]
struct AppState {
    db: FirestoreDb,
    client_id: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let port = match std::env::var("PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(_) => 8080,
    };

    let client_id = std::env::var("CLIENT_ID").unwrap();

    let appstate = {
        let db = FirestoreDb::new("esc2025").await.unwrap();

        AppState { db, client_id }
    };

    println!("Starting esc-api on port {port}...");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(appstate.clone()))
            .service(post_user)
            .service(get_user)
            .service(post_ranking)
            .service(get_ranking)
            .service(get_score)
            .service(get_lock)
            .service(post_lock)
            .service(health)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
