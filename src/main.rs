use actix_web::{
    body::BoxBody, get, http::header::ContentType, middleware::Logger, post, web, App, HttpRequest,
    HttpResponse, HttpServer, Responder,
};
use env_logger::Env;
use firestore::FirestoreDb;
use serde::{Deserialize, Serialize};

const RANKINGS_COLLECTION: &'static str = "rankings";
const ENDRESULT_COLLECTION: &'static str = "endresult";
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

#[get("/ranking/{name}")]
async fn get_ranking(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();

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
        .one(ENDRESULT_ID)
        .await
        .unwrap()
        .expect("ranking not found");

    let body = serde_json::to_string(&result).unwrap();
    HttpResponse::Ok().body(body)
}

struct AppState {
    db: FirestoreDb,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let port = match std::env::var("PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(_) => 8080,
    };

    let db = FirestoreDb::new("esc-api-384517").await.unwrap();
    println!("Starting esc-api on port {}...", port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(AppState { db: db.clone() }))
            .service(post_ranking)
            .service(get_ranking)
            .service(result)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
