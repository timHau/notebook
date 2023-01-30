mod cell;
mod notebook;
mod topology;
mod traits;

use actix_cors::Cors;
use actix_web::{http, post, web, App, HttpResponse, HttpServer, Responder};
use cell::{Cell, CellType};
use notebook::Notebook;
use serde::Deserialize;
use serde_json::json;
use tracing_subscriber;

#[derive(Deserialize)]
struct EvalRequest {
    notebook: Notebook,
    cell_uuid: String,
}

#[post("/eval")]
async fn eval(req: web::Json<EvalRequest>) -> impl Responder {
    let mut notebook = req.notebook.clone();
    let cell_uuid = req.cell_uuid.clone();
    notebook.eval(&cell_uuid);
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

#[post("/")]
async fn index() -> impl Responder {
    let notebook = Notebook::new();
    println!("Notebook created: {}", notebook.uuid);
    web::Json(notebook)
}

#[derive(Deserialize)]
struct UpdateRequest {
    cell_uuid: String,
    content: String,
    notebook: Notebook,
}

#[post("/update")]
async fn update(req: web::Json<UpdateRequest>) -> impl Responder {
    let notebook = req.notebook.clone();
    // match notebook.update_cell(&req.cell_uuid, &req.content) {
    //     Ok(_) => HttpResponse::Ok().json(notebook),
    //     Err(e) => {
    //         HttpResponse::BadRequest().json(json!({ "status": "error", "message": e.to_string() }))
    //     }
    // }
    HttpResponse::Ok().json(notebook)
}

#[derive(Deserialize)]
struct CellAdddRequest {
    notebook: Notebook,
    cell_type: CellType,
}

#[post("/add")]
async fn add(req: web::Json<CellAdddRequest>) -> impl Responder {
    let mut notebook = req.notebook.clone();
    // notebook.add_cell(req.cell_type.clone());
    HttpResponse::Ok().json(notebook)
}

#[derive(Deserialize)]
struct SaveRequest {
    notebook: Notebook,
    path: String,
}

#[post("/save")]
async fn save(req: web::Json<SaveRequest>) -> impl Responder {
    let notebook = req.notebook.clone();
    let path = req.path.clone();
    println!("Saving notebook to {:?}", notebook);
    match notebook.save(&path) {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "ok" })),
        Err(e) => {
            HttpResponse::BadRequest().json(json!({ "status": "error", "message": e.to_string() }))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(index)
            .service(eval)
            .service(update)
            .service(add)
            .service(save)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
