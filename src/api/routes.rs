use crate::{
    api::{state::State, ws::Ws},
    core::{cell::LocalValue, kernel_client::KernelClientMsg, notebook::Notebook},
};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::info;

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

pub fn notebook_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(reorder_cells);
}

#[derive(Serialize)]
pub struct EvalResponse {
    pub result: EvalResult,
}

pub type EvalResult = HashMap<String, HashMap<String, LocalValue>>; // cell_uuid -> (var_name -> var_value)

#[derive(Deserialize)]
struct WsQuery {
    #[serde(rename = "notebookUuid")]
    notebook_uuid: String,
}

#[get("/")]
async fn ws_route(
    req: HttpRequest,
    state: web::Data<State>,
    query: web::Query<WsQuery>,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let notebook_uuid = query.notebook_uuid.clone();
    info!("Opening websocket for notebook {}", notebook_uuid);
    let notebooks = state.open_notebooks.lock();
    if notebooks.is_err() {
        return Ok(HttpResponse::InternalServerError()
            .json(json!({ "status": "error", "message": "Could not lock notebooks" })));
    }
    let notebooks = notebooks.unwrap();
    let notebook = match notebooks.get(&notebook_uuid) {
        Some(notebook) => notebook,
        None => return Ok(HttpResponse::NotFound().json(json!({ "status": "Notebook not found" }))),
    };

    let kernel_sender = match state.kernel_sender.lock() {
        Ok(kernel_sender) => kernel_sender,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError()
                .json(json!({ "status": "error", "message": "Could not lock kernel sender" })));
        }
    };
    let kernel_sender = kernel_sender.clone();

    let ws_socket = Ws::new(notebook);
    let (addr, res) = ws::WsResponseBuilder::new(ws_socket, &req, stream).start_with_addr()?;

    let kernel_init_msg = KernelClientMsg::InitWs(notebook.uuid.clone(), addr.clone());
    kernel_sender.send(kernel_init_msg).unwrap();

    Ok(res)
}

pub fn ws_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(ws_route);
}
