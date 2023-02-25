use crate::{api::state::State, core::notebook::Notebook};
use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

#[get("/")]
pub async fn index(state: web::Data<State>) -> impl Responder {
    let open_notebooks = state.open_notebooks.lock();
    if open_notebooks.is_err() {
        return HttpResponse::InternalServerError()
            .json(json!({ "status": "error", "message": "Could not lock notebooks" }));
    }

    let mut open_notebooks = open_notebooks.unwrap();
    if !open_notebooks.is_empty() {
        let notebook = open_notebooks
            .get(&open_notebooks.keys().next().unwrap().clone())
            .unwrap();
        return HttpResponse::Ok().json(notebook);
    }

    let kernel_sender = match state.kernel_sender.lock() {
        Ok(kernel_sender) => kernel_sender,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "status": "error", "message": "Could not lock kernel sender" }));
        }
    };

    let notebook = Notebook::new(kernel_sender.clone());
    let notebook_uuid = notebook.uuid.clone();
    open_notebooks.insert(notebook_uuid, notebook.clone());

    HttpResponse::Ok().json(notebook)
}
