use actix_web::{
    body::BoxBody, get, http::header::ContentType, post, web, App, HttpRequest, HttpResponse,
    HttpServer, Responder, Result,
};
use firestore::FirestoreDb;
use serde::{Deserialize, Serialize};

const COLLECTION_NAME: &'static str = "esc-api";

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

fn _test_ranking() -> Ranking {
    Ranking {
        name: "test".to_string(),
        countries: vec![
            "Germany".to_string(),
            "France".to_string(),
            "Italy".to_string(),
        ],
    }
}

#[post("/ranking")]
async fn post_ranking(ranking: web::Json<Ranking>, data: web::Data<AppState>) -> impl Responder {
    let r = ranking.0;

    data.db
        .fluent()
        .update()
        .in_col(COLLECTION_NAME)
        .document_id(&r.name)
        .object(&r)
        .execute::<Ranking>()
        .await
        .unwrap();

    HttpResponse::Ok()
}

#[get("/ranking/{name}")]
async fn get_ranking(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();

    let ranking = data
        .db
        .fluent()
        .select()
        .by_id_in(COLLECTION_NAME)
        .obj::<Ranking>()
        .one(name)
        .await
        .unwrap()
        .expect("ranking not found");

    ranking
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct EndResult {
    countries: Vec<String>,
}

#[get("/result")]
async fn result() -> Result<impl Responder> {
    let result = EndResult {
        countries: vec![
            "Germany".to_string(),
            "France".to_string(),
            "Italy".to_string(),
        ],
    };

    Ok(web::Json(result))
}

struct AppState {
    db: FirestoreDb,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = match std::env::var("PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(_) => 8080,
    };

    let db = FirestoreDb::new("esc-api-384517").await.unwrap();

    println!("Starting esc-api on port {}...", port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: db.clone() }))
            .service(post_ranking)
            .service(get_ranking)
            .service(result)
    })
    .bind(("localhost", port))?
    .run()
    .await
}
