mod notebook;

use actix_cors::Cors;
use actix_web::{get, http, post, web, App, HttpServer, Responder};
use notebook::Notebook;
use pyo3::prelude::*;
use serde::Deserialize;

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
async fn index() -> impl Responder {
    let notebook = Notebook::new();
    web::Json(notebook)
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

        App::new().wrap(cors).service(index).service(eval)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
