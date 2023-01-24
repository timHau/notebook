mod cell;
mod notebook;
mod traits;

use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{get, http, post, web, App, HttpResponse, HttpServer, Responder};
use notebook::Notebook;
use pyo3::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct EvalRequest {
    code: String,
}

#[post("/eval")]
async fn eval(req: web::Json<EvalRequest>) -> impl Responder {
    let res: PyResult<()> = Python::with_gil(|py| {
        let sys = py.import("sys")?;
        let version = sys.getattr("version")?.extract::<String>()?;

        println!("Python version {}", version);

        let fun: Py<PyAny> = PyModule::from_code(py, &req.code, "", "")?
            .getattr("add")?
            .into();

        let res = fun.call1(py, (1, 2))?;
        println!("Result: {}", res);

        Ok(())
    });

    match res {
        Ok(_) => "Hello world!".to_string(),
        Err(e) => e.to_string(),
    }
}

#[get("/")]
async fn index(state: web::Data<AppState>) -> impl Responder {
    let notebook = Notebook::new();
    state.add_notebook(notebook.clone());
    web::Json(notebook)
}

#[derive(Deserialize)]
struct SaveRequest {
    uuid: String,
    path: String,
}

#[post("/save")]
async fn save(req: web::Json<SaveRequest>, state: web::Data<AppState>) -> impl Responder {
    let notebook = state.find_notebook(&req.uuid);
    match notebook {
        Some(n) => {
            n.save(&req.path).unwrap();
            HttpResponse::Ok().json(json!({ "status": "ok" }))
        }
        None => {
            println!("Notebook not found: {}, {:?}", req.uuid, state.notebooks);
            HttpResponse::NotFound().json(json!({ "status": "not found" }))
        }
    }
}

#[derive(Debug)]
struct AppState {
    notebooks: Mutex<Vec<Notebook>>,
}

impl AppState {
    fn add_notebook(&self, notebook: Notebook) {
        self.notebooks.lock().unwrap().push(notebook);
    }

    fn find_notebook(&self, uuid: &str) -> Option<Notebook> {
        let notebooks = self.notebooks.lock().unwrap();
        notebooks.iter().find(|n| n.uuid == uuid).cloned()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            notebooks: Mutex::new(Vec::new()),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(AppState::default()))
            .wrap(cors)
            .service(index)
            .service(eval)
            .service(save)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
