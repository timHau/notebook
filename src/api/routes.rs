use super::ws::Ws;
use crate::{api::state::State, core::notebook::Notebook};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use serde::Deserialize;
use serde_json::json;
use tracing::info;

#[get("/")]
pub async fn index(state: web::Data<State>) -> impl Responder {
    let mut open_notebooks = state.open_notebooks.lock().unwrap();
    if !open_notebooks.is_empty() {
        let notebook = open_notebooks
            .get(&open_notebooks.keys().next().unwrap().clone())
            .unwrap();
        return HttpResponse::Ok().json(notebook);
    }

    let notebook = Notebook::new();
    open_notebooks.insert(notebook.uuid.clone(), notebook.clone());

    HttpResponse::Ok().json(notebook)
}

#[derive(Deserialize)]
struct EvalRequest {
    #[serde(rename = "notebookUuid")]
    notebook_uuid: String,

    #[serde(rename = "cellUuid")]
    cell_uuid: String,
}

#[post("/eval")]
async fn eval(req: web::Json<EvalRequest>, state: web::Data<State>) -> impl Responder {
    let notebook_uuid = req.notebook_uuid.clone();
    let cell_uuid = req.cell_uuid.clone();

    let mut notebooks = state.open_notebooks.lock().unwrap();
    let notebook = match notebooks.get_mut(&notebook_uuid) {
        Some(notebook) => notebook,
        None => return HttpResponse::NotFound().json(json!({ "status": "Notebook not found" })),
    };

    let nb = notebook.clone();
    let cell = match notebook.get_cell_mut(&cell_uuid) {
        Some(cell) => cell,
        None => return HttpResponse::NotFound().json(json!({ "status": "Cell not found" })),
    };

    match nb.eval_cell(cell) {
        Ok(_) => (),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "status": "error", "message": e.to_string() }))
        }
    }

    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

pub fn notebook_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(eval);
}

#[get("/")]
async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    info!("Websocket connection established");
    let res = ws::start(Ws {}, &req, stream);
    res
}

pub fn ws_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(ws_route);
}
