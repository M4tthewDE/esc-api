use actix_web::{
    body::BoxBody, get, http::header::ContentType, post, web, App, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
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
async fn post_ranking(
    ranking: web::Json<Ranking>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if !authorized(data.secret.clone(), req) {
        return HttpResponse::Unauthorized().finish();
    }
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
async fn get_ranking(
    path: web::Path<String>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if !authorized(data.secret.clone(), req) {
        return HttpResponse::Unauthorized().finish();
    }
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
async fn result(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    if !authorized(data.secret.clone(), req) {
        return HttpResponse::Unauthorized().finish();
    }

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

fn authorized(secret: String, req: HttpRequest) -> bool {
    let header = req.headers().get("Authorization");
    return match header {
        None => false,
        Some(s) => s.to_str().unwrap() == secret,
    };
}

struct AppState {
    db: FirestoreDb,
    secret: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = match std::env::var("PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(_) => 8080,
    };

    let secret = match std::env::var("SECRET") {
        Ok(s) => s,
        Err(_) => "hunter2".to_string(),
    };

    let db = FirestoreDb::new("esc-api-384517").await.unwrap();
    println!("Starting esc-api on port {}...", port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db: db.clone(),
                secret: secret.clone(),
            }))
            .service(post_ranking)
            .service(get_ranking)
            .service(result)
    })
    .bind(("localhost", port))?
    .run()
    .await
}
