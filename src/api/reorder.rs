use crate::api::state::State;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct ReorderRequest {
    #[serde(rename = "notebookUuid")]
    notebook_uuid: String,

    #[serde(rename = "newOrder")]
    new_order: Vec<String>,
}

#[post("/reorder")]
async fn reorder_cells(req: web::Json<ReorderRequest>, state: web::Data<State>) -> impl Responder {
    let notebook_uuid = req.notebook_uuid.clone();
    let new_order = req.new_order.clone();

    let notebooks = state.open_notebooks.lock();
    if notebooks.is_err() {
        return HttpResponse::InternalServerError()
            .json(json!({ "status": "error", "message": "Could not lock notebooks" }));
    }
    let mut notebooks = notebooks.unwrap();
    let notebook = match notebooks.get_mut(&notebook_uuid) {
        Some(notebook) => notebook,
        None => return HttpResponse::NotFound().json(json!({ "status": "Notebook not found" })),
    };
    notebook.reorder_cells(&new_order);

    HttpResponse::Ok().json(json!({ "status": "ok" }))
}
