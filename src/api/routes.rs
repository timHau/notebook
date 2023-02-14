use crate::{api::state::State, core::notebook::Notebook};
use actix_web::{get, post, web, HttpResponse, Responder};
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
        info!("{:#?}", notebook);
        return HttpResponse::Ok().json(notebook);
    }

    let kernel = state.kernel.lock().unwrap();
    let notebook = Notebook::new(kernel.clone());
    info!("{:#?}", notebook);
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

    let mut notebook = state.open_notebooks.lock().unwrap();
    let notebook = match notebook.get_mut(&notebook_uuid) {
        Some(notebook) => notebook,
        None => return HttpResponse::NotFound().json(json!({ "status": "not found" })),
    };

    let mut nb = notebook.clone();
    let cell = match nb.get_cell_mut(&cell_uuid) {
        Some(cell) => cell,
        None => return HttpResponse::NotFound().json(json!({ "status": "not found" })),
    };

    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

pub fn notebook_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(eval);
}
